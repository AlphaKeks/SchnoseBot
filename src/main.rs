#![allow(dead_code)]

use {
	std::env,
	anyhow::Result,
	dotenv::dotenv,
	simple_logger::SimpleLogger,
	serenity::{
		async_trait, Client,
		prelude::{Context, GatewayIntents, EventHandler},
		model::{application::interaction::Interaction, channel::Message, prelude::Ready},
	},
};

mod commands;
mod db;
mod events;
mod util;

#[tokio::main]
async fn main() -> Result<()> {
	// load environment variables
	dotenv()?;

	// instantiate logging
	SimpleLogger::new()
		.with_colors(true)
		.with_level(log::LevelFilter::Warn)
		.init()?;

	// create metadata
	let token = env::var("DISCORD_TOKEN")?;
	let icon_url = env::var("ICON_URL")?;
	let schnose = Schnose::new(token, icon_url);

	// create serenity client
	let mut client =
		Client::builder(&schnose.token, schnose.intents).event_handler(schnose).await?;

	// connect to discord
	client.start().await?;

	return Ok(());
}

#[derive(Debug, Clone)]
pub struct Schnose {
	pub token: String,
	pub intents: GatewayIntents,
	pub icon_url: String,
}

impl Schnose {
	fn new(token: String, icon_url: String) -> Self {
		Self {
			token,
			icon_url,
			intents: GatewayIntents::GUILDS
				| GatewayIntents::GUILD_MEMBERS
				| GatewayIntents::GUILD_MESSAGES
				| GatewayIntents::GUILD_MESSAGE_REACTIONS
				| GatewayIntents::MESSAGE_CONTENT,
		}
	}
}

#[async_trait]
impl EventHandler for Schnose {
	async fn ready(&self, ctx: Context, ready: Ready) {
		let _ = events::ready::handle(self, ctx, ready).await;
	}

	async fn message(&self, ctx: Context, msg: Message) {
		let _ = events::message::handle(self, ctx, msg).await;
	}

	async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
		match interaction {
			Interaction::ApplicationCommand(slash_command) => {
				let _ = events::slash_command::handle(self, ctx, slash_command).await;
			},
			_ => unimplemented!(),
		}
	}
}
