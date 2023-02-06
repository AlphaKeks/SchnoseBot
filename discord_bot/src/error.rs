use log::{error, info, warn};

/// Global Error type for the entire crate.
#[derive(Debug, Clone)]
#[allow(clippy::upper_case_acronyms)]
pub enum Error {
	/// Some unknown error occurred.
	Unknown,

	/// Some custom edge-case error that doesn't deserve it's own eblame_use: blame_usernt.
	Custom(String),

	/// Used for the static map cache / invalid user input.
	MapNotGlobal,

	/// Failed to access the database.
	DatabaseAccess,

	/// Failed to update an entry in the database.
	DatabaseUpdate,

	/// The user didn't specify a `SteamID` and also has no database entries for it.
	MissingSteamID {
		/// If the user @mention'd somebody, the blame is not on them. If they didn't specify any
		/// `player` argument though, we need to tell them about `/setsteam`.
		blame_user: bool,
	},

	/// The user didn't specify a `Mode` and also has no database entries for it.
	MissingMode,

	/// Failed to parse JSON.
	ParseJSON,

	/// User Input was out of range.
	InputOutOfRange,

	/// An error from the `gokz_rs` crate.
	GOKZ(String),
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}",
			match self {
				Error::Unknown => "Some unknown error occurred.",
				Error::Custom(msg) => msg,
				Error::MapNotGlobal => "The map you specified is not global.",
				Error::DatabaseAccess => "Failed to access the database.",
				Error::DatabaseUpdate => "Failed to update an entry in the database.",
				Error::MissingSteamID { blame_user } => if *blame_user {
                    "You didn't specify a SteamID and also didn't set it with `/setsteam`. Please specify a SteamID or save yours with `/setsteam`."
                } else {
                    "The user you @mention'd didn't save their SteamID in my database."
                }
				Error::MissingMode => "You didn't specify a mode and also didn't set your preference with `/mode`. Please specify one or use `/mode` to set a preference.",
				Error::ParseJSON => "Failed to parse JSON.",
				Error::InputOutOfRange => "Your input was out of range. Please provide some realistic values.",
				Error::GOKZ(msg) => msg
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

impl From<gokz_rs::prelude::Error> for Error {
	fn from(value: gokz_rs::prelude::Error) -> Self {
		Self::GOKZ(value.msg)
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
