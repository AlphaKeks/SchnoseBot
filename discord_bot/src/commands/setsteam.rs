use {
	crate::{
		error::{Error, Result},
		Context, State,
	},
	gokz_rs::SteamID,
	sqlx::QueryBuilder,
};

/// Save your SteamID in the bot's database.
///
/// This command will save your `SteamID` in its database for later use. Since many commands have \
/// a `player` parameter you probably don't want to specify that over and over again. Instead you \
/// can use this command and the bot will remember your choice in the future.
#[tracing::instrument(skip(ctx), fields(user = ctx.author().tag()))]
#[poise::command(slash_command, on_error = "Error::handle_command")]
pub async fn setsteam(
	ctx: Context<'_>,
	#[description = "Your SteamID, e.g. `STEAM_1:1:161178172`"] steam_id: String,
) -> Result<()> {
	ctx.defer().await?;

	let steam_id = SteamID::new(&steam_id)?;

	let (name, id) = {
		let author = ctx.author();
		(&author.name, *author.id.as_u64())
	};

	let table = &ctx.config().mysql_table;

	match ctx.find_user_by_id(id).await {
		// User already has a database entry => modify current one
		Ok(user) => {
			// :tf:
			if user.steam_id.as_ref() == Some(&steam_id) {
				ctx.say("You already have this SteamID set.")
					.await?;
				return Ok(());
			}

			let mut query = QueryBuilder::new(format!(r#"UPDATE {table} SET steam_id = "#));

			query
				.push_bind(steam_id.to_string())
				.push(" WHERE discord_id = ")
				.push_bind(id);

			query
				.build()
				.execute(ctx.database())
				.await?;
		}
		// We failed to get the user's database entry. Why?
		Err(why) => match why {
			// This is not supposed to happen! Return with an error.
			why @ (Error::DatabaseAccess | Error::DatabaseUpdate) => return Err(why),
			// The user simply has no entry yet => create a new one
			_ => {
				let mut query = QueryBuilder::new(format!(
					r#"
					INSERT INTO {table}
					    (name, discord_id, steam_id)
					"#
				));

				query.push_values([(name, id, steam_id)], |mut query, (name, id, steam_id)| {
					query
						.push_bind(name)
						.push_bind(id)
						.push_bind(steam_id.to_string());
				});

				query
					.build()
					.execute(ctx.database())
					.await?;
			}
		},
	};

	ctx.say(format!("Successfully set SteamID `{steam_id}` for <@{id}>!"))
		.await?;

	Ok(())
}
