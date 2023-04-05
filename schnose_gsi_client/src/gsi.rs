use {
	crate::{config::Config, server},
	color_eyre::Result,
	gokz_rs::{schnose_api, MapIdentifier, Mode, SteamID, Tier},
	schnose_gsi::{GSIConfigBuilder, GSIServer, Result as GSIResult, ServerHandle, Subscription},
	serde::{Deserialize, Serialize},
	std::{
		sync::{Arc, Mutex},
		time::Duration,
	},
	tokio::sync::mpsc::UnboundedSender,
	tracing::{debug, error, info},
};

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
	pub tier: Option<Tier>,
}

pub fn run_server(
	axum_sender: UnboundedSender<server::Payload>,
	config: Config,
) -> GSIResult<ServerHandle> {
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
	let axum_sender = Arc::new(axum_sender);
	let last_info = Arc::new(Mutex::new(Option::<server::Payload>::None));

	gsi_server.add_async_event_listener(move |event| {
		let gokz_client = Arc::clone(&gokz_client);
		// let gsi_sender = Arc::clone(&gsi_sender);
		let axum_sender = Arc::clone(&axum_sender);
		let last_info = Arc::clone(&last_info);
		let api_key = config.schnose_api_key.clone();

		Box::pin(async move {
			// debug!("New GSI Event: {event:#?}");
			info!("New GSI Event.");

			let mut info = {
				// Get info from last event
				let mut last_info_guard = match last_info.lock() {
					Ok(guard) => guard,
					Err(why) => {
						error!("Failed to acquire Mutex guard.");
						debug!("Failed to acquire Mutex guard: {why:?}");
						return;
					}
				};

				let new_info = server::Payload {
					map_name: event
						.map
						.as_ref()
						.map(|map| {
							if map.name.contains('/') {
								let (_, map_name) = map
									.name
									.rsplit_once('/')
									.unwrap_or_default();
								String::from(map_name)
							} else {
								map.name.clone()
							}
						})
						.unwrap_or_else(|| String::from("unknown map")),
					map_tier: None,
					mode: event
						.player
						.as_ref()
						.and_then(|player| {
							player
								.clan
								.as_ref()
								.and_then(|clan_tag| {
									if clan_tag.contains(' ') {
										// [SKZ Beginner] Player Name
										let (mode, _rank) = clan_tag.split_once(' ')?;
										mode.replace('[', "")
											.parse::<Mode>()
											.ok()
									} else {
										// [SKZ] Player Name
										clan_tag
											.replace(['[', ']'], "")
											.parse::<Mode>()
											.ok()
									}
								})
						}),
					steam_id: event
						.player
						.as_ref()
						.map(|player| player.steam_id),
				};

				// If `last_info` does not yet exist, initialize it with the current info.
				let last_info = match &*last_info_guard {
					Some(info) => info.clone(),
					None => {
						*last_info_guard = Some(new_info.clone());
						new_info.clone()
					}
				};

				let same_map = new_info.map_name == last_info.map_name;
				let same_mode = new_info.mode == last_info.mode;
				let same_player = new_info.steam_id == last_info.steam_id;

				// If the new info is the same as the last one, we don't want to do anything.
				if same_map && same_mode && same_player {
					return;
				} else {
					debug!("last map: {}", last_info.map_name);
					debug!("new map: {}", new_info.map_name);
					debug!("last mode: {:?}", last_info.mode);
					debug!("new mode: {:?}", new_info.mode);
					debug!("last player: {:?}", last_info.steam_id);
					debug!("new player: {:?}", new_info.steam_id);
				}

				*last_info_guard = Some(new_info.clone());

				new_info
			};

			info.map_tier = match schnose_api::get_map(
				&MapIdentifier::Name(info.map_name.clone()),
				&gokz_client,
			)
			.await
			{
				Ok(map) => Some(map.tier),
				Err(_) => None,
			};

			// Send the update to the Axum backend.
			match axum_sender.send(info.clone()) {
				Ok(()) => {
					info!("[TO Axum] Info sent successfully.");
					debug!("[TO Axum] Info sent successfully: {info:#?}");
				}
				Err(why) => {
					error!("Failed to send info to Axum.");
					debug!("Failed to send info to Axum: {why:?}");
				}
			};

			// Also notfiy SchnoseAPI.
			if let Some(steam_id) = info.steam_id {
				let csgo_report = CSGOReport {
					player_name: event
						.player
						.as_ref()
						.map(|player| player.name.clone())
						.unwrap_or_else(|| String::from("unknown")),
					steam_id,
					mode: info.mode,
					map: Some(Map { name: info.map_name, tier: info.map_tier }),
				};

				match post_to_schnose_api(csgo_report.clone(), api_key, &gokz_client).await {
					Ok(()) => {
						info!("[TO SchnoseAPI] Report sent successfully.");
						debug!("[TO SchnoseAPI] Report sent successfully: {csgo_report:#?}");
					}
					Err(why) => {
						error!("Failed to send report to SchnoseAPI.");
						debug!("Failed to send report to SchnoseAPI: {why:?}");
					}
				};
			}
		})
	});

	info!("Listening for CS:GO events on port {}.", config.gsi_port);

	gsi_server.run()
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
			error!("POST failed.");
			debug!("POST failed: {why:#?}");
			Err(why.into())
		}
	}
}
