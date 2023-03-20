//! `MySQL` module for the bot's database.

use {
	gokz_rs::{Mode, SteamID},
	sqlx::FromRow,
};

/// `MySQL` schema for a user row.
#[derive(Debug, Clone, FromRow)]
pub struct UserSchema {
	pub name: String,
	pub discord_id: u64,
	pub steam_id: Option<String>,
	pub mode: Option<u8>,
}

/// Parsed version of [`UserSchema`].
#[derive(Debug, Clone)]
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
			steam_id: value
				.steam_id
				.and_then(|steam_id| SteamID::new(steam_id).ok()),
			mode: value
				.mode
				.and_then(|mode| Mode::try_from(mode).ok()),
		}
	}
}
