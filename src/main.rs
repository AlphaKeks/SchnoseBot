mod commands;
mod event_handler;
mod util;

use std::env;

use serenity::framework::StandardFramework;
use serenity::model::application::interaction::Interaction;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::GatewayIntents;
use serenity::Client;
use serenity::{
	async_trait,
	prelude::{Context, EventHandler},
};

const DEFAULT_ICON_URL: &str = "https://cdn.discordapp.com/attachments/981130651094900756/981130719537545286/churchOfSchnose.png";

pub struct Schnose {
	pub token: String,
	pub intents: GatewayIntents,
	pub icon: String,
}

impl Schnose {
	pub fn new(token: String, icon_url: String) -> Self {
		Schnose {
			token,
			icon: icon_url,
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
	// 1. registers commands to Discord's API
	// 2. sets the bot's activity status
	// 3. prints connection confirmation message
	async fn ready(&self, ctx: Context, ready: Ready) {
		event_handler::ready::handle(ctx, ready).await;
	}

	// 1. handles commands
	async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
		event_handler::interaction_create::handle(&self, ctx, interaction).await;
	}

	// 1. replies to "bing?"
	async fn message(&self, ctx: Context, msg: Message) {
		event_handler::message::handle(ctx, msg).await;
	}
}

#[tokio::main]
async fn main() {
	// load .env file
	dotenv::dotenv().expect("Failed to load environment variables.");

	// instantiate metadata
	let schnose = match env::var("DISCORD_TOKEN") {
		Err(why) => panic!("No Discord API Token found. {:#?}", why),
		Ok(token) => match env::var("ICON") {
			Ok(icon) => Schnose::new(token, icon),
			Err(_) => Schnose::new(token, DEFAULT_ICON_URL.to_owned()),
		},
	};

	// instantiate new serenity client
	let mut schnose = Client::builder(&schnose.token, schnose.intents)
		.framework(StandardFramework::new())
		.event_handler(schnose)
		.await
		.expect("Failed to create new client.");

	// instantiate logging
	if let Err(why) = simple_logger::SimpleLogger::new()
		.with_colors(true)
		.with_level(log::LevelFilter::Warn)
		.init()
	{
		println!("Failed to initialize logging {:#?}", why)
	}

	// start the client
	if let Err(why) = schnose.start().await {
		panic!("Failed to start client. {:#?}", why);
	}
}
