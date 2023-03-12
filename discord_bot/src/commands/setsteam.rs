use {
	crate::{
		error::{Error, Result},
		Context, State,
	},
	gokz_rs::SteamID,
	log::trace,
};

/// Save your `SteamID` in the bot's database.
///
/// This command will associate the `SteamID` you specify with your Discord `UserID` for later \
/// use. This is very helpful on commands such as `/wr` or `/profile` where the bot needs to \
/// target a specific player.
#[poise::command(slash_command, on_error = "Error::handle_command")]
pub async fn setsteam(
	ctx: Context<'_>,
	#[description = "Your SteamID, e.g. `STEAM_1:1:161178172`"] steam_id: String,
) -> Result<()> {
	trace!("[/setsteam ({})]", ctx.author().tag());
	trace!("> `steam_id`: {steam_id:?}");
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

			sqlx::query(&format!(
				r#"UPDATE {table} SET steam_id = "{steam_id}" WHERE discord_id = {id}"#,
			))
			.execute(ctx.database())
			.await?;
		}
		// We failed to get the user's database entry. Why?
		Err(why) => match why {
			// This is not supposed to happen! Return with an error.
			why @ (Error::DatabaseAccess | Error::DatabaseUpdate) => return Err(why),
			// The user simply has no entry yet => create a new one
			_ => {
				sqlx::query(&format!(
					r#"INSERT INTO {table} (name, discord_id, steam_id) VALUES("{name}", {id}, "{steam_id}")"#,
				))
				.execute(ctx.database())
				.await?;
			}
		},
	};

	ctx.say(format!("Successfully set SteamID `{steam_id}` for <@{id}>!"))
		.await?;

	Ok(())
}
