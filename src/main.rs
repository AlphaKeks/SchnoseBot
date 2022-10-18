mod commands;
mod events;
mod util;

use std::env;

use serenity::builder::CreateEmbed;
use serenity::framework::standard::macros::group;
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

pub enum SchnoseCommand {
	Message(String),
	Embed(CreateEmbed),
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
	async fn ready(&self, ctx: Context, ready: Ready) {
		events::ready::ready(ctx, ready).await
	}

	async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
		events::commands::interaction_create(ctx, interaction).await
	}

	async fn message(&self, ctx: Context, message: Message) {
		events::bing::message(ctx, message).await
	}
}

#[group]
struct General;

#[tokio::main]
async fn main() {
	dotenv::dotenv().expect("Failed to load env file");

	let token = env::var("DISCORD_TOKEN").expect("no discord token found");

	let framework = StandardFramework::new();

	let intents = GatewayIntents::GUILD_MESSAGES
		| GatewayIntents::DIRECT_MESSAGES
		| GatewayIntents::MESSAGE_CONTENT;

	let mut client = Client::builder(&token, intents)
		.framework(framework)
		.event_handler(Handler)
		.await
		.expect("fuck you thats why");

	if let Err(why) = simple_logger::SimpleLogger::new()
		.with_level(log::LevelFilter::Warn)
		.init()
	{
		println!("Failed to initialize logging: {:#?}", why);
	}

	if let Err(why) = client.start().await {
		panic!("client crashed: {:#?}", why);
	}
}
