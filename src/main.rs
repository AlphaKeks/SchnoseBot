use std::env;

mod schnose;
use schnose::BotData;

mod commands;
mod db;
mod events;
mod util;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	// setup .env
	if let Err(why) = dotenv::dotenv() {
		eprintln!("Failed to initialize environment variables.");
		panic!("{why}");
	}

	// setup logging
	env_logger::init();

	// setup client
	let Ok(token) = env::var("DISCORD_TOKEN") else {
		log::error!("Missing `DISCORD_TOKEN` environment variable.");
		panic!();
	};
	let state = BotData::new(token, "users").await?;

	let mut client = serenity::Client::builder(&state.token, state.intents)
		.event_handler(state)
		.await?;

	// connect to Discord
	log::info!("connecting to discord...");
	client.start().await?;

	return Ok(());
}
