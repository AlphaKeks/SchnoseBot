//! The global [`Error`] and [`Result`] types used across the entire crate.

use log::{error, info, warn};

pub type Result<T> = std::result::Result<T, Error>;

/// Global `Error` type for the entire crate.
#[derive(Debug, Clone)]
#[allow(clippy::upper_case_acronyms)]
pub enum Error {
	/// Some unknown error occurred.
	Unknown,

	/// Some custom edge-case error that doesn't deserve it's own enum variant.
	Custom(String),

	/// Used for the static map cache / invalid user input.
	MapNotGlobal,

	/// Failed to access the database.
	DatabaseAccess,

	/// Failed to update an entry in the database.
	DatabaseUpdate,

	/// Failed to find any database entries for a given user.
	NoDatabaseEntries,

	/// The user didn't specify a `SteamID` and also has no database entries for it.
	MissingSteamID {
		/// If the user @mention'd somebody, tell them that that user doesn't have any database
		/// entries. If they didn't specify any `player` argument though, we need to tell them
		/// about `/setsteam`.
		blame_user: bool,
	},

	/// The user didn't specify a `Mode` and also has no database entries for it.
	MissingMode,

	/// No SteamID or Name was found in the database and `player` param wasn't specified.
	NoPlayerInfo,

	/// Failed to parse JSON.
	ParseJSON,

	/// User Input was out of range.
	InputOutOfRange,

	/// An error from the [`gokz_rs`] crate.
	GOKZ(String),

	/// No records were found for a given query.
	NoRecords,

	/// Failed to restart the bot's process.
	BotRestart,

	/// Failed to `git pull`.
	GitPull,

	/// Failed to clean target dir.
	CleanTargetDir,

	/// Failed to compile.
	Build,

	/// A command that only works on a Guild was called somewhere else.
	NoGuild { reason: String },
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(
			match self {
				Error::Unknown => "Some unknown error occurred.",
				Error::Custom(msg) => msg,
				Error::MapNotGlobal => "Map is not global.",
				Error::DatabaseAccess => "Failed to access the database.",
				Error::DatabaseUpdate => "Failed to update an entry in the database.",
				Error::NoDatabaseEntries => "No database entries found.",
				Error::MissingSteamID { blame_user } => if *blame_user {
					"You didn't specify a SteamID and also didn't set it with `/setsteam`. Please specify a SteamID or save yours with `/setsteam`."
				} else {
					"The user you @mention'd didn't save their SteamID in my database."
				}
				Error::MissingMode => "You didn't specify a mode and also didn't set your preference with `/mode`. Please specify one or use `/mode` to set a preference.",
				Error::NoPlayerInfo => "You didn't specify a `player` parameter and don't have any database entries. Please specify a `player` or set your SteamID via `/setsteam`.",
				Error::ParseJSON => "Failed to parse JSON.",
				Error::InputOutOfRange => "Your input was out of range. Please provide some realistic values.",
				Error::GOKZ(msg) => msg,
				Error::NoRecords => "No records found.",
				Error::BotRestart => "Failed to restart.",
				Error::GitPull => "Failed to pull from GitHub.",
				Error::CleanTargetDir => "Failed to clean build directory.",
				Error::Build => "Failed to compile.",
				Error::NoGuild { reason } => return f.write_fmt(format_args!("You can only call this command on a server{}.", reason))
			}
		)
	}
}

impl std::error::Error for Error {}

impl From<serenity::Error> for Error {
	fn from(value: serenity::Error) -> Self {
		match value {
			serenity::Error::Json(why) => {
				error!("JSON Error {why:?}");
				Self::ParseJSON
			}
			serenity::Error::NotInRange(param, value, min, max) => {
				warn!("User Input (`{value}`) for `{param}` out of range (`{min}` - `{max}`)");
				Self::InputOutOfRange
			}
			why => {
				warn!("Error occurred: {why:?}");
				Self::Unknown
			}
		}
	}
}

impl From<gokz_rs::Error> for Error {
	fn from(value: gokz_rs::Error) -> Self {
		Self::GOKZ(value.to_string())
	}
}

impl From<sqlx::Error> for Error {
	fn from(value: sqlx::Error) -> Self {
		warn!("DB ERROR `{value:?}`");
		match value {
			sqlx::Error::Database(why) => {
				warn!("{why:?}");
				Self::DatabaseAccess
			}
			sqlx::Error::RowNotFound => {
				warn!("{}", value.to_string());
				Self::NoDatabaseEntries
			}
			_ => Self::DatabaseAccess,
		}
	}
}

impl From<color_eyre::Report> for Error {
	fn from(value: color_eyre::Report) -> Self {
		Self::Custom(value.to_string())
	}
}

impl Error {
	pub async fn handle_command(error: poise::FrameworkError<'_, crate::GlobalState, Error>) {
		error!("Slash Command failed. {error:?}");

		let (content, ephemeral) = match &error {
			poise::FrameworkError::Command { error, .. } => (error.to_string(), false),
			poise::FrameworkError::ArgumentParse { input, .. } => (
				format!(
					"You provided invalid input. {}",
					if let Some(input) = input { input } else { "" }
				),
				false,
			),
			poise::FrameworkError::CommandStructureMismatch { description, .. } => {
				error!("{description}");
				(String::from("Incorrect command structure."), false)
			}
			poise::FrameworkError::CooldownHit { remaining_cooldown, .. } => {
				(
					format!(
						"This command is currently on cooldown. Please wait another {:.2} seconds before trying again.", remaining_cooldown.as_secs_f64()
					),
					true
				)
			}
			poise::FrameworkError::MissingBotPermissions { missing_permissions, .. } => {
				error!("{missing_permissions}");
				(
					String::from("The bot is missing permissions for this action. Please contact the server owner and kindly ask them to give the bot the required permissions."),
					false
				)
			}
			poise::FrameworkError::MissingUserPermissions { missing_permissions, .. } => {
				(
					if let Some(perms) = missing_permissions {
						format!("You are missing the `{perms}` permissions for this command.")
					} else {
						String::from("You are missing the required permissions for this command.")
					},
					true
				)
			}
			poise::FrameworkError::NotAnOwner { .. } => {
				(String::from("This command requires you to be the owner of the bot."), true)
			}
			why => {
				error!("{why:?}");
				(String::from("Failed to execute command."), true)
			}
		};

		if let Some(ctx) = &error.ctx() {
			if let Err(why) = ctx
				.send(|reply| {
					reply
						.ephemeral(ephemeral)
						.content(&content)
				})
				.await
			{
				error!("Failed to respond to slash command. {why:?}");
			}

			info!("Handled error with `{content}`.");
		}
	}
}
