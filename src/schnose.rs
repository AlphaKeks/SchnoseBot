use {
	std::env,
	crate::{events, db::UserSchema},
	bson::doc,
	gokz_rs::prelude::*,
	serde::{Serialize, Deserialize},
	serenity::{
		async_trait,
		model::prelude::{Ready, interaction::Interaction, User, Message},
		prelude::{GatewayIntents, EventHandler, Context},
	},
	mongodb::Collection,
};

/// Custom Error type so I don't have to keep typing the same error messages everywhere
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum SchnoseErr {
	Custom(String),
	Parse,
	// whether to blame the user for not specifying a SteamID or not
	MissingSteamID(bool),
	MissingMode,
	FailedDB,
}

impl std::fmt::Display for SchnoseErr {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let s = match self {
			Self::Custom(msg) => &msg,
			Self::Parse => "",
			Self::MissingMode => "You need to specify a mode or set a default one via `/mode`.",
			Self::MissingSteamID(blame_user) => {
				if *blame_user {
					"You need to specify a player or save your SteamID in schnose's database via `/setsteam`."
				} else {
					"The player you mentioned didn't save their SteamID in schnose's database. Tell them to use `/setsteam`!"
				}
			},
			Self::FailedDB => "Failed to access database.",
		};

		return write!(f, "{}", s);
	}
}

/// Global data for initializing a new bot instance
#[derive(Debug, Clone)]
pub(crate) struct BotData {
	pub token: String,
	pub intents: GatewayIntents,
	pub db: Collection<UserSchema>,
	pub req_client: reqwest::Client,
	pub icon: String,
}

impl BotData {
	pub async fn new(token: String, collection: &str) -> anyhow::Result<Self> {
		let mongo_url = env::var("MONGO_URL")?;
		let mongo_options = mongodb::options::ClientOptions::parse(mongo_url).await?;
		let mongo_client = mongodb::Client::with_options(mongo_options)?;
		let collection = mongo_client.database("gokz").collection(collection);

		let req_client = reqwest::Client::new();
		let icon = env::var("ICON_URL").unwrap_or(
			String::from("https://cdn.discordapp.com/attachments/981130651094900756/981130719537545286/churchOfSchnose.png")
		);

		return Ok(Self {
			token,
			intents: GatewayIntents::GUILDS
				| GatewayIntents::GUILD_MEMBERS
				| GatewayIntents::GUILD_MESSAGES
				| GatewayIntents::GUILD_MESSAGE_REACTIONS
				| GatewayIntents::MESSAGE_CONTENT,
			db: collection,
			req_client,
			icon,
		});
	}
}

#[async_trait]
impl EventHandler for BotData {
	/// Gets triggered once on startup
	async fn ready(&self, ctx: Context, ready: Ready) {
		if let Err(why) = events::ready::handle(self, ctx, ready).await {
			log::error!("Failed to respond to `ready` event.\n\n{:?}", why);
		}
	}

	/// Gets triggered on every new interaction;
	/// currently only /slash_commands are being handled
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

	/// Gets triggered on every message the bot _can_ see (on any server).
	async fn message(&self, ctx: Context, msg: Message) {
		if let Err(why) = events::message::handle(ctx, msg).await {
			log::error!("Failed to respond to `message` event.\n\n{:?}", why);
		}
	}
}

/// A lot of commands have a `player` parameter which is used to determine who the user is
/// targetting (e.g. on `/pb`). Regex is being used to disambiguate between the different kinds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum Target {
	None,
	Name(String),
	SteamID(SteamID),
	Mention(Mention),
}

impl Target {
	/// Create a new `Target` from user input. The intended way to use this is in combination with
	/// calling `.get` on the `GlobalState` passed into every command.
	/// ```
	/// let user_input = state.get::<String>("player");
	/// let target = Target::from(user_input);
	/// ```
	pub fn from(input: Option<String>) -> Self {
		let Some(input) = input else {
			return Target::None;
		};
		if let Ok(steam_id) = SteamID::new(&input) {
			return Target::SteamID(steam_id);
		}
		if let Ok(mention) = input.parse::<Mention>() {
			return Target::Mention(mention);
		}
		return Target::Name(input);
	}

	/// Turn a `Target` into a [PlayerIdentifier](gokz_rs::prelude::PlayerIdentifier)
	pub async fn to_player(
		self,
		user: &User,
		collection: &Collection<UserSchema>,
	) -> Result<PlayerIdentifier, SchnoseErr> {
		// `blame_user` determines the kind of error message sent later
		let (user_id, blame_user) = match self {
			Target::SteamID(steam_id) => return Ok(PlayerIdentifier::SteamID(steam_id)),
			Target::Name(name) => return Ok(PlayerIdentifier::Name(name)),
			Target::Mention(user_id) => (user_id.into(), false),
			Target::None => (*user.id.as_u64(), true),
		};

		// Search database for the user's Discord User ID
		match collection.find_one(doc! { "discordID": user_id.to_string() }, None).await {
			// Database connection successful
			Ok(document) => {
				// User has an entry in the database
				if let Some(entry) = document {
					// `steamID` field in the database entry is not null
					if let Some(steam_id) = entry.steamID {
						return Ok(PlayerIdentifier::SteamID(
							SteamID::new(&steam_id).expect(
								"This should never be invalid. If it is, fix the database.",
							),
						));
					}
				}
				return Err(SchnoseErr::MissingSteamID(blame_user));
			},
			// Database connection failed
			Err(why) => {
				log::error!("[{}]: {} => {:?}", file!(), line!(), why);
				return Err(SchnoseErr::FailedDB);
			},
		}
	}
}

/// Helper type to handle Discord's @mention's easer
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub(crate) struct Mention(pub u64);

impl std::str::FromStr for Mention {
	type Err = SchnoseErr;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		use regex::Regex;

		let regex =
			Regex::new(r#"^<@[0-9]+>$"#).expect("If it compiles once, it will always compile.");

		if !regex.is_match(s) {
			return Err(SchnoseErr::Parse);
		}

		let user_id = str::replace(s, "<@", "");
		let user_id = str::replace(&user_id, ">", "");
		let user_id = user_id.parse::<u64>().expect("This should be a valid u64.");

		return Ok(Mention(user_id));
	}
}

impl std::fmt::Display for Mention {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		return write!(f, "<@{}>", self.0);
	}
}

impl From<u64> for Mention {
	fn from(id: u64) -> Self {
		return Self(id);
	}
}

impl Into<u64> for Mention {
	fn into(self) -> u64 {
		return self.0;
	}
}
