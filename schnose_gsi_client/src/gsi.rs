use crate::config::Config;
use color_eyre::Result;
use gokz_rs::{schnose_api, MapIdentifier, Mode, SteamID, Tier};
use schnose_gsi::{GSIConfigBuilder, GSIServer, Subscription};
use serde::{Deserialize, Serialize};
use std::{
	sync::{Arc, Mutex},
	time::Duration,
};
use tokio::sync::mpsc::UnboundedSender;
use tracing::{error, info, warn};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CSGOReport {
	pub player_name: String,
	pub steam_id: SteamID,
	pub mode: Option<Mode>,
	pub map: Option<Map>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Map {
	pub name: String,
	pub tier: Tier,
}

pub async fn run_server(gsi_sender: UnboundedSender<CSGOReport>, config: Config) {
	let mut config_builder = GSIConfigBuilder::new("schnose-gsi-client");

	config_builder
		.heartbeat(Duration::from_secs(1))
		.subscribe_multiple([
			Subscription::Map,
			Subscription::PlayerID,
		]);

	let gsi_config = config_builder.build();

	let mut gsi_server = GSIServer::new(gsi_config, config.gsi_port);

	let cfg_dir_is_fake = !config.csgo_cfg_path.exists();
	let cfg_dir_is_cwd = config
		.csgo_cfg_path
		.as_os_str()
		.is_empty();

	if cfg_dir_is_fake || cfg_dir_is_cwd {
		gsi_server
			.install()
			.expect("Failed to locate CS:GO directory.");
	} else {
		gsi_server
			.install_into(&config.csgo_cfg_path)
			.unwrap_or_else(|_| {
				panic!("Failed to install config into `{}`.", config.csgo_cfg_path.display())
			});
	}

	let gokz_client = Arc::new(gokz_rs::Client::new());
	let gsi_sender = Arc::new(gsi_sender);
	let last_info = Arc::new(Mutex::new(None));

	gsi_server.add_async_event_listener(move |event| {
		let gokz_client = Arc::clone(&gokz_client);
		let gsi_sender = Arc::clone(&gsi_sender);
		let last_info = Arc::clone(&last_info);
		let api_key = config.schnose_api_key.clone();

		Box::pin(async move {
			info!("New GSI Event: {event:#?}");

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

			let map = match event.map {
				Some(map) => {
					let Ok(map) = schnose_api::get_map(&MapIdentifier::from(map.name.clone()), &gokz_client).await else {
						return warn!("Failed to fetch map `{map:?}`.");
					};

					Some(Map {
						name: map.name,
						tier: map.tier
					})
				}
				None => None
			};

			let csgo_report = CSGOReport {
				player_name: player.name.clone(),
				steam_id: player.steam_id,
				mode,
				map
			};

			{
				let mut last_info_lock = match last_info.lock() {
					Ok(lock) => lock,
					Err(why) => return error!("Failed to acquire Mutex: {why:?}"),
				};

				// If Map/Mode/Player haven't changed, return early.
				if (*last_info_lock).as_ref() == Some(&csgo_report) {
					return;
				} else {
					*last_info_lock = Some(csgo_report.clone());
				}
			}

			// Otherwise notify the GUI thread and SchnoseAPI
			info!("Info has changed! New: {csgo_report:#?}");

			match gsi_sender.send(csgo_report.clone()) {
				Ok(()) => info!("Info sent successfully."),
				Err(why) => error!("Failed sending info: {why:?}"),
			};

			match post_to_schnose_api(csgo_report, api_key, &gokz_client).await {
				Ok(()) => info!("POSTed successfully."),
				Err(why) => error!("Failed to POST: {why:?}"),
			};
		})
	});

	info!("Listening for CS:GO events on port {}.", config.gsi_port);

	gsi_server
		.run()
		.await
		.expect("Failed to run GSI server.");
}

async fn post_to_schnose_api(
	csgo_report: CSGOReport,
	api_key: String,
	client: &gokz_rs::Client,
) -> Result<()> {
	match client
		.post("https://schnose.xyz/api/twitch_info")
		.json(&csgo_report)
		.header("x-schnose-auth-key", api_key)
		.send()
		.await
		.map(|res| res.error_for_status())
	{
		Ok(Ok(res)) => {
			info!("POST successful: {res:#?}");
			Ok(())
		}
		Ok(Err(why)) | Err(why) => {
			error!("POST failed: {why:#?}");
			Err(why.into())
		}
	}
}
