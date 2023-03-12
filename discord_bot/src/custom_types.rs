//! Custom types which I couldn't find a better place for.

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

	fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
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
	pub async fn into_player(self, ctx: &Context<'_>) -> Result<PlayerIdentifier> {
		match self {
			Self::None(user_id) => {
				if let Ok(user) = ctx.find_user_by_id(user_id).await {
					if let Some(steam_id) = user.steam_id {
						Ok(PlayerIdentifier::SteamID(steam_id))
					} else {
						Ok(PlayerIdentifier::Name(user.name))
					}
				} else {
					Ok(PlayerIdentifier::Name(ctx.author().name.clone()))
				}
			}
			Self::Mention(user_id) => ctx
				.find_user_by_id(user_id)
				.await
				.map(|user| {
					if let Some(steam_id) = user.steam_id {
						Ok(PlayerIdentifier::SteamID(steam_id))
					} else {
						Ok(PlayerIdentifier::Name(user.name))
					}
				})?,
			Self::SteamID(steam_id) => Ok(PlayerIdentifier::SteamID(steam_id)),
			Self::Name(ref name) => {
				if let Ok(user) = ctx.find_user_by_name(name).await {
					if let Some(steam_id) = user.steam_id {
						Ok(PlayerIdentifier::SteamID(steam_id))
					} else {
						Ok(PlayerIdentifier::Name(user.name))
					}
				} else {
					let player = schnose_api::get_player(
						PlayerIdentifier::Name(name.to_owned()),
						ctx.gokz_client(),
					)
					.await?;

					Ok(PlayerIdentifier::SteamID(player.steam_id))
				}
			}
		}
	}
}
