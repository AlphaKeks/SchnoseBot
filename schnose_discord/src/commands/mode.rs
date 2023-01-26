use {
	super::{handle_err, DBModeChoice},
	crate::{
		GlobalStateAccess, database,
		SchnoseError::{self, *},
	},
	log::{debug, trace, info, error},
};

/// Save your favorite mode for later use.
#[poise::command(slash_command, on_error = "handle_err")]
pub async fn mode(
	ctx: crate::Context<'_>,
	#[description = "KZT/SKZ/VNL"] mode: DBModeChoice,
) -> Result<(), SchnoseError> {
	ctx.defer().await?;

	trace!("[/mode] mode: `{}`", &mode);

	let query =
		format!("SELECT * FROM discord_users WHERE discord_id = {}", ctx.author().id.as_u64());

	debug!("query: {}", &query);

	match sqlx::query_as::<_, database::UserSchema>(&query)
		.fetch_one(ctx.database())
		.await
	{
		Ok(data) => {
			// user has data => change current `mode`
			info!("Updating DB entry");
			debug!("Old data: {:?}", data);
			let query = format!(
				"UPDATE discord_users SET mode = {} WHERE discord_id = {}",
				if mode as u8 == 0 { String::from("NULL") } else { (mode as u8).to_string() },
				ctx.author().id.as_u64()
			);

			match sqlx::query(&query)
				.execute(ctx.database())
				.await
			{
				Ok(_result) => {
					ctx.say(format!("Successfully updated Mode. New value: `{}`", mode))
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
					"INSERT INTO discord_users (name, discord_id, mode) VALUES (\"{}\", {}, {})",
					&ctx.author().name,
					ctx.author().id.as_u64(),
					if mode as u8 == 0 { String::from("NULL") } else { (mode as u8).to_string() },
				);

				match sqlx::query(&query)
					.execute(ctx.database())
					.await
				{
					Ok(_result) => {
						ctx.say(format!(
							"Successfully set Mode `{}` for <@{}>.",
							mode,
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
