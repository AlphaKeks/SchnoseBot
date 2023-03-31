use {
	csgo_gsi::{GSIConfigBuilder, GSIServer, Subscription},
	gokz_rs::{schnose_api, MapIdentifier, Mode, SteamID, Tier},
	serde::Serialize,
	serde_json::Value as JsonValue,
	std::{sync::mpsc::Sender, time::Duration},
	tracing::{error, info},
};

/// This struct holds the relevant information about the game that we will send to SchnoseAPI and
/// display on the desktop client.
#[derive(Debug, Clone, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Info {
	pub player_name: String,
	pub steam_id: SteamID,
	pub mode: Option<Mode>,
	pub map: Option<(String, Tier)>,
}

/// Start the GSI server that will listen for updates from CS:GO and send them to the GUI thread.
pub async fn run(cfg_path: String, port: u16, tx: Sender<Info>) {
	let gsi_config = GSIConfigBuilder::new("schnose-csgo-watcher")
		.heartbeat(Duration::from_secs(1))
		.subscribe_multiple([
			Subscription::Map,
			Subscription::PlayerID,
		])
		.build();

	let mut server = GSIServer::new(gsi_config, port);

	server
		.install_into(cfg_path)
		.expect("Failed to create config file.");

	let gokz_client = gokz_rs::BlockingClient::new();

	server.add_listener(move |event| {
		info!("GSI Event: {event:#?}");

		let player = event
			.player
			.as_ref()
			.expect("There is always a player.");

		let mode = player
			.clan
			.as_ref()
			.and_then(|clan_tag| {
				clan_tag
					.replace(['[', ']'], "")
					.split_once(' ')
					.and_then(|(mode, _rank)| mode.parse::<Mode>().ok())
			});

		let steam_id = player
			.steam_id
			.parse::<SteamID>()
			.expect("If CS:GO ever sends an invalid SteamID, shoot me.");

		let map = event
			.map
			.as_ref()
			.and_then(|map| get_map_blocking(map.name.clone(), &gokz_client).ok());

		let info = Info {
			player_name: player.name.clone(),
			steam_id,
			mode,
			map,
		};

		info!("Sending info: {info:#?}");

		if let Err(why) = tx.send(info) {
			error!("Failed sending message: {why:#?}");
		}
	});

	info!("Listening for GSI events.");

	server
		.run()
		.await
		.expect("Failed to run GSI server.");
}

fn get_map_blocking(
	map_identifier: impl Into<MapIdentifier>,
	client: &gokz_rs::BlockingClient,
) -> gokz_rs::Result<(String, Tier)> {
	let map_identifier = map_identifier.into();
	let base_url = schnose_api::BASE_URL;
	let url = format!("{base_url}/maps/{map_identifier}");

	let json = client
		.get(url)
		.send()?
		.json::<JsonValue>()?;

	let map = json
		.get("result")
		.ok_or(gokz_rs::Error::Custom("Failed to deserialize result."))?;

	let map_name = map
		.get("name")
		.ok_or(gokz_rs::Error::Custom("Failed to deserialize map name."))?
		.as_str()
		.ok_or(gokz_rs::Error::Custom("Failed to deserialize map name as string."))?
		.to_owned();

	let map_tier =
		map.get("tier")
			.ok_or(gokz_rs::Error::Custom("Failed to deserialize map tier."))?
			.as_u64()
			.ok_or(gokz_rs::Error::Custom("Failed to deserialize map tier as number."))? as u8;

	Ok((map_name, map_tier.try_into()?))
}
