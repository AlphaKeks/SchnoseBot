//! `MySQL` module for the bot's database.

use {
	gokz_rs::{Mode, SteamID},
	serde::{Deserialize, Serialize},
	sqlx::FromRow,
};

/// `MySQL` representation of a user row in the database.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserSchema {
	pub name: String,
	pub discord_id: u64,
	pub steam_id: Option<String>,
	pub mode: Option<u8>,
}

/// Parsed version of [`UserSchema`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
	pub name: String,
	pub discord_id: u64,
	pub steam_id: Option<SteamID>,
	pub mode: Option<Mode>,
}

impl From<UserSchema> for User {
	fn from(value: UserSchema) -> Self {
		Self {
			name: value.name,
			discord_id: value.discord_id,
			steam_id: {
				if let Some(steam_id) = value.steam_id {
					SteamID::new(&steam_id).ok()
				} else {
					None
				}
			},
			mode: {
				if let Some(mode_id) = value.mode {
					Mode::try_from(mode_id).ok()
				} else {
					None
				}
			},
		}
	}
}
