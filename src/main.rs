//! Discord Bot using [serenity.rs](https://docs.rs/serenity-rs) for interfacing with the [GlobalAPI](https://kztimerglobal.com/swagger/index.html?urls.primaryName=V2).
//! The bot supports most stat-related ingame commands, like [/wr](crate::commands::wr) for example.
//! You can use [/setsteam](crate::commands::setsteam) to link your Steam account to your Discord account.
//! Use [/mode](crate::commands::mode) to save your preferred gamemode so you don't have to specify it on every command
//! where the gamemode would be relevant.
//!
//! Any bug reports / suggestions can either be submitted as [an issue on GitHub](https://github.com/AlphaKeks/SchnoseBot/issues) or via Discord DM
//! `@AlphaKeks#9826`.

use std::env;

mod schnose;
use schnose::BotData;

mod commands;
mod db;
mod events;
mod gokz;
mod util;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	// Load a `.env` file to retreive relevant information (like the Discord API token) later
	if let Err(why) = dotenv::dotenv() {
		panic!("Failed to initialize environment variables.\n{}", why);
	}

	// Initialize Logger for debugging and tracking reported issues
	env_logger::init();

	// Initialize some global data that will live for the entirety of the program
	let Ok(token) = env::var("DISCORD_TOKEN") else {
		log::error!("Missing `DISCORD_TOKEN` environment variable.");
		panic!();
	};
	let state = BotData::new(token, "users").await?;

	// Initialize serenity client to connect to Discord
	let mut client = serenity::Client::builder(&state.token, state.intents)
		.event_handler(state)
		.await?;

	// Try to connect to Discord
	log::info!("connecting to discord...");
	client.start().await?;

	return Ok(());
}
