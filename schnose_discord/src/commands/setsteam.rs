use {
	super::handle_err,
	crate::{
		GlobalStateAccess, database,
		SchnoseError::{self, *},
	},
	log::{debug, trace, info, error},
	gokz_rs::prelude::*,
};

/// Save your SteamID for later use.
#[poise::command(slash_command, on_error = "handle_err")]
pub async fn setsteam(
	ctx: crate::Context<'_>,
	#[description = "Your SteamID, e.g. `STEAM_1:1:161178172`"] steam_id: String,
) -> Result<(), SchnoseError> {
	ctx.defer().await?;

	trace!("[/setsteam] steam_id: `{}`", &steam_id);

	if !SteamID::test(&steam_id) {
		return Err(SchnoseError::Custom(format!("`{}` is not a valid SteamID.", &steam_id)));
	}

	let query =
		format!("SELECT * FROM discord_users WHERE discord_id = {}", ctx.author().id.as_u64());

	debug!("query: {}", &query);

	match sqlx::query_as::<_, database::UserSchema>(&query)
		.fetch_one(ctx.database())
		.await
	{
		Ok(data) => {
			// user has data => change current `steam_id`
			info!("Updating DB entry");
			debug!("Old data: {:?}", data);
			let query = format!(
				"UPDATE discord_users SET steam_id = \"{}\" WHERE discord_id = {}",
				&steam_id,
				ctx.author().id.as_u64()
			);

			match sqlx::query(&query).execute(ctx.database()).await {
				Ok(_result) => {
					ctx.say(format!("Successfully updated SteamID. New value: `{}`", steam_id))
						.await?;
					Ok(())
				},
				Err(why) => {
					error!("Failed to update DB entry: {:?}", why);
					Err(DatabaseUpdate)
				},
			}
		},
		Err(why) => match why {
			// user has no data yet => create new row
			sqlx::Error::RowNotFound => {
				let query = format!(
					"INSERT INTO discord_users (name, discord_id, steam_id) VALUES (\"{}\", {}, \"{}\")",
					&ctx.author().name,
					ctx.author().id.as_u64(),
					&steam_id
				);

				match sqlx::query(&query).execute(ctx.database()).await {
					Ok(_result) => {
						ctx.say(format!(
							"Successfully set SteamID `{}` for <@{}>.",
							steam_id,
							ctx.author().id.as_u64()
						))
						.await?;
						Ok(())
					},
					Err(why) => {
						error!("Failed to create DB entry: {:?}", why);
						Err(DatabaseUpdate)
					},
				}
			},
			// something has gone very wrong
			_ => Err(DatabaseAccess),
		},
	}
}
