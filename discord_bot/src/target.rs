//! Custom type for the `player` parameter on many commands.

use {
	crate::{
		error::{Error, Result},
		Context, State,
	},
	gokz_rs::{schnose_api, PlayerIdentifier, SteamID},
	regex::Regex,
};

/// Enum for `player` parameters on commands.
#[derive(Debug, Clone)]
pub enum Target {
	/// The user didn't specify a target -> we take their `UserID`.
	None(u64),

	/// The user @mention'd somebody -> we take that `UserID`.
	Mention(u64),

	/// The user put in a valid `SteamID` -> we take that.
	SteamID(SteamID),

	/// The user specified none of the above. We interpret that as a name.
	Name(String),
}

impl std::str::FromStr for Target {
	type Err = Error;

	fn from_str(s: &str) -> Result<Self> {
		if let Ok(steam_id) = SteamID::new(s) {
			return Ok(Self::SteamID(steam_id));
		}

		if Regex::new(r#"<@[0-9]+>"#)
			.unwrap()
			.is_match(s)
		{
			if let Ok(user_id) = s
				.replace("<@", "")
				.replace('>', "")
				.parse::<u64>()
			{
				return Ok(Self::Mention(user_id));
			}
		}

		Ok(Self::Name(s.to_owned()))
	}
}

impl Target {
	pub async fn parse_input(input: Option<String>, ctx: &Context<'_>) -> Result<PlayerIdentifier> {
		if let Some(input) = input {
			input.parse::<Self>()?
		} else {
			Self::None(*ctx.author().id.as_u64())
		}
		.into_player(ctx)
		.await
	}

	pub async fn into_player(self, ctx: &Context<'_>) -> Result<PlayerIdentifier> {
		match self {
			Self::None(user_id) => {
				if let Ok(user) = ctx.find_user_by_id(user_id).await {
					if let Some(steam_id) = user.steam_id {
						Ok(steam_id.into())
					} else {
						Ok(user.name.into())
					}
				} else {
					Ok(ctx.author().name.clone().into())
				}
			}
			Self::Mention(user_id) => ctx
				.find_user_by_id(user_id)
				.await
				.map(|user| {
					if let Some(steam_id) = user.steam_id {
						Ok(steam_id.into())
					} else {
						Ok(user.name.into())
					}
				})?,
			Self::SteamID(steam_id) => Ok(PlayerIdentifier::SteamID(steam_id)),
			Self::Name(name) => {
				if let Ok(user) = ctx.find_user_by_name(&name).await {
					if let Some(steam_id) = user.steam_id {
						Ok(steam_id.into())
					} else {
						Ok(user.name.into())
					}
				} else {
					Ok(schnose_api::get_player(PlayerIdentifier::Name(name), ctx.gokz_client())
						.await
						.map(|player| player.steam_id)?
						.into())
				}
			}
		}
	}
}
