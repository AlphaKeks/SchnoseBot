use serenity::model::prelude::Message;

use {
	std::env,
	crate::{events, db::UserSchema, util::Mention},
	bson::doc,
	gokz_rs::prelude::*,
	serde::{Serialize, Deserialize},
	serenity::{
		async_trait,
		model::prelude::{Ready, interaction::Interaction, User},
		prelude::{GatewayIntents, EventHandler, Context},
	},
	mongodb::Collection,
};

#[derive(Debug, Clone)]
pub(crate) struct BotData {
	pub token: String,
	pub intents: GatewayIntents,
	pub db: Collection<UserSchema>,
}

impl BotData {
	pub async fn new(token: String, collection: &str) -> anyhow::Result<Self> {
		let mongo_url = env::var("MONGO_URL")?;
		let mongo_options = mongodb::options::ClientOptions::parse(mongo_url).await?;
		let mongo_client = mongodb::Client::with_options(mongo_options)?;
		let collection = mongo_client.database("gokz").collection(collection);

		return Ok(Self {
			token,
			intents: GatewayIntents::GUILDS
				| GatewayIntents::GUILD_MEMBERS
				| GatewayIntents::GUILD_MESSAGES
				| GatewayIntents::GUILD_MESSAGE_REACTIONS
				| GatewayIntents::MESSAGE_CONTENT,
			db: collection,
		});
	}
}

#[async_trait]
impl EventHandler for BotData {
	async fn ready(&self, ctx: Context, ready: Ready) {
		if let Err(why) = events::ready::handle(self, ctx, ready).await {
			log::error!("Failed to respond to `ready` event.\n\n{:?}", why);
		}
	}

	async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
		match interaction {
			Interaction::ApplicationCommand(slash_command) => {
				if let Err(why) = events::slash_commands::handle(self, ctx, slash_command).await {
					log::error!("Failed to respond to slash command.\n\n{:?}", why);
				}
			},
			unknown_interaction => {
				log::warn!("encountered unknown interaction: `{:?}`", unknown_interaction)
			},
		}
	}

	async fn message(&self, ctx: Context, msg: Message) {
		if let Err(why) = events::message::handle(ctx, msg).await {
			log::error!("Failed to respond to `message` event.\n\n{:?}", why);
		}
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum Target {
	None,
	Name(String),
	SteamID(SteamID),
	Mention(u64),
}

impl Target {
	pub fn from(input: Option<String>) -> Self {
		let Some(input) = input else {
			return Target::None;
		};
		if let Ok(steam_id) = SteamID::new(&input) {
			return Target::SteamID(steam_id);
		}
		if let Some(mention) = Mention::from(&input) {
			return Target::Mention(mention.0);
		}
		return Target::Name(input);
	}

	pub async fn to_player(
		self,
		user: &User,
		collection: &Collection<UserSchema>,
	) -> Result<PlayerIdentifier, String> {
		let (user_id, blame_user) = match self {
			Target::SteamID(steam_id) => return Ok(PlayerIdentifier::SteamID(steam_id)),
			Target::Name(name) => return Ok(PlayerIdentifier::Name(name)),
			Target::Mention(user_id) => (user_id, false),
			Target::None => (*user.id.as_u64(), true),
		};

		match collection.find_one(doc! { "discordID": user_id.to_string() }, None).await {
			Ok(document) => {
				if let Some(entry) = document {
					if let Some(steam_id) = entry.steamID {
						return Ok(PlayerIdentifier::SteamID(
							SteamID::new(&steam_id).expect(
								"This should never be invalid. If it is, fix the database.",
							),
						));
					}
				}

				let reply = if blame_user {
					"You need to specify a player or save your SteamID in schnose's database via `/setsteam`."
				} else {
					"The player you mentioned didn't save their SteamID in schnose's database. Tell them to use `/setsteam`!"
				};

				return Err(String::from(reply));
			},
			Err(why) => {
				log::error!("[{}]: {} => {:?}", file!(), line!(), why);
				return Err(String::from("Failed to access database."));
			},
		}
	}
}
