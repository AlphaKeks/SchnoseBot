use {
	crate::database::schemas::UserSchema,
	log::error,
	bson::doc,
	gokz_rs::prelude::*,
	mongodb::Collection,
	serenity::{builder::CreateEmbed, model::user::User},
};

/// Custom Error type for this crate
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) enum SchnoseError {
	UserInput(String),
	GOKZ(String),
	Parse(String),
	MissingSteamID(bool),
	MissingMode(bool),
	MissingDBEntry(bool),
	DBAccess,
	DBUpdate,
	Defer,
	Custom(String),
}

impl std::fmt::Display for SchnoseError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let result = match self {
			SchnoseError::UserInput(invalid_input) => format!("`{}` is not a valid input.", invalid_input),
			SchnoseError::GOKZ(err_msg) => format!("{}", err_msg),
			SchnoseError::Parse(source) => format!("Failed to parse {}.", source),
			SchnoseError::MissingSteamID(blame_user) => format!("{}",
				if *blame_user {
					"You need to either specify a player or save your SteamID in schnose's database via `/setsteam`."
				} else {
					"The player you specified didn't save their SteamID in schnose's database. Tell them to use `/setsteam`!"
				}
			),
			SchnoseError::MissingMode(blame_user) => format!("{}",
				if *blame_user {
					"You need to either specify a mode or save your preferred mode in schnose's database via `/mode`."
				} else {
					"The player you specified didn't save their preferred mode in schnose's database. Tell them to use `/mode`!"
				}
			) ,
			SchnoseError::MissingDBEntry(blame_user) => format!("{}",
				if *blame_user {
					"You don't have any database entries yet."
				} else {
					"The player you specified doesn't have any database entries yet."
				}
			),
			SchnoseError::DBAccess => format!("Failed to access database. Please report this incident to <@291585142164815873>."),
			SchnoseError::DBUpdate => format!("Failed to update database. Please report this incident to <@291585142164815873>."),
			SchnoseError::Defer => format!("Failed to defer interaction. Please report this incident to <@291585142164815873>."),
			SchnoseError::Custom(msg) => format!("{}", msg)
		};

		return write!(f, "{}", result);
	}
}

impl From<gokz_rs::prelude::Error> for SchnoseError {
	fn from(error: gokz_rs::prelude::Error) -> Self {
		return Self::GOKZ(error.tldr);
	}
}

#[derive(Debug, Clone)]
pub(crate) enum InteractionResponseData {
	Message(String),
	Embed(CreateEmbed),
}

impl From<&str> for InteractionResponseData {
	fn from(s: &str) -> Self {
		return Self::Message(s.to_owned());
	}
}

impl From<String> for InteractionResponseData {
	fn from(s: String) -> Self {
		return Self::Message(s);
	}
}

impl From<CreateEmbed> for InteractionResponseData {
	fn from(embed: CreateEmbed) -> Self {
		return Self::Embed(embed);
	}
}

impl From<SchnoseError> for InteractionResponseData {
	fn from(error: SchnoseError) -> Self {
		return Self::Message(error.to_string());
	}
}

pub(crate) type InteractionResult = Result<InteractionResponseData, SchnoseError>;

/// Helper type to handle Discord's @mention's easer
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub(crate) struct Mention(pub u64);

impl std::str::FromStr for Mention {
	type Err = SchnoseError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		use regex::Regex;

		let regex =
			Regex::new(r#"^<@[0-9]+>$"#).expect("If it compiles once, it will always compile.");

		if !regex.is_match(s) {
			return Err(SchnoseError::UserInput(s.to_owned()));
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

/// A lot of commands have a `player` parameter which is used to determine who the user is
/// targetting (e.g. on `/pb`). Regex is being used to disambiguate between the different kinds.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) enum Target {
	None,
	Name(String),
	SteamID(SteamID),
	Mention(Mention),
}

impl Target {
	/// Turn a `Target` into a [PlayerIdentifier](gokz_rs::prelude::PlayerIdentifier)
	pub async fn to_player(
		self,
		user: &User,
		collection: &Collection<UserSchema>,
	) -> Result<PlayerIdentifier, SchnoseError> {
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
				return Err(SchnoseError::MissingSteamID(blame_user));
			},
			// Database connection failed
			Err(why) => {
				error!("{}", why);
				return Err(SchnoseError::DBAccess);
			},
		}
	}
}

impl From<Option<String>> for Target {
	/// Create a new `Target` from user input. The intended way to use this is in combination with
	/// calling `.get` on the `InteractionState` passed into every command.
	/// ```
	/// let user_input = state.get::<String>("player");
	/// let target = Target::from(user_input);
	/// ```
	fn from(input: Option<String>) -> Self {
		let Some(input) = input else {
			return Self::None;
		};

		if let Ok(steam_id) = SteamID::new(&input) {
			return Self::SteamID(steam_id);
		}

		if let Ok(mention) = input.parse::<Mention>() {
			return Target::Mention(mention);
		}

		return Target::Name(input);
	}
}
