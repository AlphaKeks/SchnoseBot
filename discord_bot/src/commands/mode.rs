use {
	super::DBModeChoice,
	crate::{error::Error, Context, State},
	gokz_rs::prelude::*,
	log::trace,
};

#[poise::command(slash_command, on_error = "Error::handle_command")]
pub async fn mode(
	ctx: Context<'_>, #[description = "KZT/SKZ/VNL"] mode: DBModeChoice,
) -> Result<(), Error> {
	ctx.defer().await?;
	trace!("[/mode] mode: `{mode}`");

	let mode = Mode::try_from(mode);
	let mode_id = mode
		.as_ref()
		.map(|mode| (*mode as u8).to_string())
		.unwrap_or_else(|_| String::from("NULL"));

	let (name, id) = {
		let author = ctx.author();
		(&author.name, *author.id.as_u64())
	};

	let table = &ctx.config().mysql_table;

	let updated = match ctx.find_by_id(id).await {
		// User already has a database entry => modify current one
		Ok(user) => {
			// :tf:
			if user.mode.as_ref() == mode.as_ref().ok() {
				ctx.say("You already have this mode set.")
					.await?;
				return Ok(());
			}

			sqlx::query(
				&format!(r#"UPDATE {table} SET mode = {mode_id} WHERE discord_id = {id}"#,),
			)
			.execute(ctx.database())
			.await?;

			true
		}
		// We failed to get the user's database entry. Why?
		Err(why) => match why {
			// This is not supposed to happen! Return with an error.
			why @ (Error::DatabaseAccess | Error::DatabaseUpdate) => return Err(why),
			// The user simply has no entry yet => create a new one
			_ => {
				if mode.is_err() {
					ctx.say("<:tf:999383331647012935>")
						.await?;
					return Ok(());
				}

				sqlx::query(&format!(
					r#"INSERT INTO {table} (name, discord_id, mode) VALUES("{name}", {id}, {mode_id})"#,
				))
				.execute(ctx.database())
				.await?;

				false
			}
		},
	};

	let reply = if let Ok(mode) = mode {
		if updated {
			format!("Successfully updated Mode for <@{id}>! New Mode: `{mode}`")
		} else {
			format!("Successfully set Mode `{mode}` for <@{id}>!")
		}
	} else {
		format!("Successfully cleared Mode for <@{id}>!")
	};

	ctx.say(reply).await?;

	Ok(())
}
