use {
	crate::{config::Config, gui::state::State},
	color_eyre::Result,
	schnose_gsi::{GSIConfigBuilder, GSIServer, Subscription},
	std::{sync::Arc, time::Duration},
	tokio::sync::Mutex,
	tracing::{debug, error, info},
};

pub fn run_server(
	state: Arc<Mutex<Option<State>>>,
	config: Arc<Mutex<Config>>,
) -> schnose_gsi::ServerHandle {
	let mut config_builder = GSIConfigBuilder::new("schnose-gsi-client");

	config_builder
		.heartbeat(Duration::from_secs(1))
		.subscribe_multiple([
			Subscription::Map,
			Subscription::PlayerID,
		]);

	let gsi_config = config_builder.build();

	let (port, detect_install_dir) = tokio::task::block_in_place(|| {
		let config = config.blocking_lock();
		let is_fake = !config.csgo_cfg_path.exists();
		let is_cwd = config
			.csgo_cfg_path
			.as_os_str()
			.is_empty();

		(config.gsi_port, is_fake || is_cwd)
	});

	let mut gsi_server = GSIServer::new(gsi_config, port);

	if detect_install_dir {
		gsi_server.install().unwrap();
	} else {
		gsi_server
			.install_into(tokio::task::block_in_place(|| {
				config
					.blocking_lock()
					.csgo_cfg_path
					.clone()
			}))
			.unwrap();
	}

	let gokz_client = Arc::new(gokz_rs::Client::new());
	let prev_event = Arc::new(Mutex::new(None));

	gsi_server.add_async_event_listener(move |event| {
		let gokz_client = Arc::clone(&gokz_client);
		let state = Arc::clone(&state);
		let config = Arc::clone(&config);
		let old_event = Arc::clone(&prev_event);

		Box::pin(async move {
			info!("New GSI Event.");
			debug!("{event:#?}");

			// Check if the new event is the same as the previous one, and return early if it is.
			// No need to re-fetch the same information.
			{
				let mut prev_event = old_event.lock().await;
				if (*prev_event).as_ref() == Some(&event) {
					return;
				}
				*prev_event = Some(event.clone());
			}

			let new_state = match State::new(event, &gokz_client).await {
				Ok(state) => state,
				Err(why) => return error!("Failed to create state from event: {why:?}"),
			};

			*state.lock().await = Some(new_state.clone());

			let schnose_api_key = config
				.lock()
				.await
				.schnose_api_key
				.clone();

			if schnose_api_key.is_empty() {
				return;
			}

			if let Err(why) = post_to_schnose_api(new_state, &schnose_api_key, &gokz_client).await {
				error!("Failed to POST to SchnoseAPI: {why:?}");
			}
		})
	});

	info!("Listening for CS:GO events on port {port}.");

	gsi_server
		.run()
		.expect("Failed to run GSI server.")
}

async fn post_to_schnose_api(state: State, api_key: &str, client: &gokz_rs::Client) -> Result<()> {
	match client
		.post("https://schnose.xyz/api/twitch_info")
		.json(&state)
		.header("x-schnose-auth-key", api_key)
		.send()
		.await
		.map(|res| res.error_for_status())
	{
		Ok(Ok(res)) => {
			info!("[SchnoseAPI] POST successful: {res:#?}");
			Ok(())
		}
		Ok(Err(why)) | Err(why) => {
			error!("[SchnoseAPI] OST failed.");
			debug!("[SchnoseAPI] OST failed: {why:#?}");
			Err(why.into())
		}
	}
}
