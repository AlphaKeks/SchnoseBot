mod commands;
mod database;
mod events;
mod formatting;
mod prelude;
mod util;

use {
	std::env,
	database::schemas::UserSchema,
	log::{info, warn, error},
	mongodb::Collection,
	serenity::{
		prelude::{GatewayIntents, EventHandler, Context},
		async_trait,
		model::prelude::{Ready, Message, interaction::Interaction},
	},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	// Load a `.env` file storing sensitive information (like API tokens)
	dotenv::dotenv().expect("Expected `.env` file.");

	// Initialize Logger for debugging
	env_logger::init();

	let Ok(token) = env::var("DISCORD_TOKEN") else {
		panic!("Missing `DISCORD_TOKEN` environment variable.");
	};

	// Create global state
	let global_state = GlobalState::new(token, "gokz", "users").await?;

	// Initialize serenity client to connect to Discord
	let mut client = serenity::Client::builder(&global_state.token, global_state.intents)
		.event_handler(global_state)
		.await?;

	info!("Connecting to Discord...");
	if let Err(why) = client.start().await {
		panic!("Failed to connect to Discord: {:?}", why);
	}

	return Ok(());
}

/// Global State object used for handling Discord events and passing static data around
#[derive(Debug, Clone)]
pub(crate) struct GlobalState {
	// Discord API auth token
	pub token: String,
	// https://discord.com/developers/docs/topics/gateway
	pub intents: GatewayIntents,
	// database for accessing user information
	pub db: Collection<UserSchema>,
	// global HTTPS client for making API calls
	pub req_client: reqwest::Client,
	// Icon URL for embed footers
	pub icon: String,
	// #7480c2
	pub colour: (u8, u8, u8),
}

impl GlobalState {
	async fn new(
		token: String,
		database_name: &str,
		collection_name: &str,
	) -> anyhow::Result<GlobalState> {
		let mongo_url = env::var("MONGO_URL")?;
		let mongo_options = mongodb::options::ClientOptions::parse(mongo_url).await?;
		let mongo_client = mongodb::Client::with_options(mongo_options)?;
		let collection =
			mongo_client.database(database_name).collection::<UserSchema>(collection_name);

		let req_client = reqwest::Client::new();

		let icon = env::var("ICON_URL").unwrap_or("https://cdn.discordapp.com/attachments/981130651094900756/981130719537545286/churchOfSchnose.png".into());

		return Ok(GlobalState {
			token,
			intents: GatewayIntents::GUILDS
				| GatewayIntents::GUILD_MEMBERS
				| GatewayIntents::GUILD_MESSAGES
				| GatewayIntents::GUILD_MESSAGE_REACTIONS
				| GatewayIntents::MESSAGE_CONTENT,
			db: collection,
			req_client,
			icon,
			colour: (116, 128, 194),
		});
	}
}

#[async_trait]
impl EventHandler for GlobalState {
	/// Gets triggered once on startup
	async fn ready(&self, ctx: Context, ready: Ready) {
		if let Err(why) = events::ready::handle(&ctx, &ready).await {
			error!("Failed to handle `READY` event: {:?}", why);
		}
	}

	/// Gets triggered on every message the bot reads
	async fn message(&self, ctx: Context, msg: Message) {
		if let Err(why) = events::message::handle(&ctx, &msg).await {
			error!("Failed to handle `MESSAGE_CREATE` event: {:?}", why);
		}
	}

	/// Gets triggered on [interactions](https://discord.com/developers/docs/interactions/receiving-and-responding).
	/// As of right now only slash commands are being handled, but Buttons and menus are planned.
	async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
		use Interaction::*;
		match interaction {
			ApplicationCommand(slash_command) => {
				if let Err(why) =
					events::interactions::slash_command::handle(&self, &ctx, &slash_command).await
				{
					error!("Failed to handle `INTERACTION_CREATE` event: {:?}", why);
				}
			},
			unknown_interaction => {
				warn!("Encountered unknown interaction `{:?}`.", unknown_interaction)
			},
		}
	}
}
