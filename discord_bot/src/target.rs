//! Custom type for the `player` parameter on many commands.

use {
	crate::{
		db,
		error::{Error, Result},
		Context, State,
	},
	gokz_rs::{global_api, schnose_api, PlayerIdentifier, SteamID},
	poise::serenity_prelude::CacheHttp,
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

	#[tracing::instrument]
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
	#[tracing::instrument]
	pub async fn parse_input(
		target: Option<String>,
		db_entry: Result<db::User>,
		ctx: &Context<'_>,
	) -> Result<PlayerIdentifier> {
		if let Some(input) = target {
			input.parse::<Self>()?
		} else {
			Self::None(*ctx.author().id.as_u64())
		}
		.into_player(db_entry, ctx)
		.await
	}

	/// Tries to turn the given [`Target`] into a [`PlayerIdentifier`] with best effort.
	#[tracing::instrument]
	pub async fn into_player(
		self,
		db_entry: Result<db::User>,
		ctx: &Context<'_>,
	) -> Result<PlayerIdentifier> {
		match self {
			Self::None(_) => {
				if let Ok(user) = db_entry {
					if let Some(steam_id) = user.steam_id {
						Ok(steam_id.into())
					} else {
						Ok(user.name.into())
					}
				} else {
					Ok(ctx.author().name.clone().into())
				}
			}
			Self::Mention(user_id) => {
				if let Ok(player_identifier) = ctx
					.find_user_by_id(user_id)
					.await
					.map(|user| {
						if let Some(steam_id) = user.steam_id {
							PlayerIdentifier::from(steam_id)
						} else {
							PlayerIdentifier::from(user.name)
						}
					}) {
					Ok(player_identifier)
				} else {
					// If the user @mention'd somebody who isn't in the database, scan the current
					// server for that member and take their username.
					let guild = ctx.guild().ok_or(Error::NoGuild {
						reason: String::from(
							" if you mention someone who doesn't have any database entries",
						),
					})?;

					let guild_member = guild
						.member(ctx.http(), user_id)
						.await?;

					Ok(guild_member.user.name.into())
				}
			}
			Self::SteamID(steam_id) => Ok(steam_id.into()),
			Self::Name(name) => {
				if let Ok(user) = ctx.find_user_by_name(&name).await {
					if let Some(steam_id) = user.steam_id {
						Ok(steam_id.into())
					} else {
						Ok(user.name.into())
					}
				} else if let Ok(player) =
					schnose_api::get_player(name.clone().into(), ctx.gokz_client()).await
				{
					Ok(player.steam_id.into())
				} else if let Ok(player) =
					global_api::get_player(name.clone().into(), ctx.gokz_client()).await
				{
					Ok(player.steam_id.into())
				} else {
					Ok(name.into())
				}
			}
		}
	}
}
