use {
	super::choices::DBModeChoice,
	crate::{
		error::{Error, Result},
		Context, State,
	},
	gokz_rs::Mode,
	log::trace,
	sqlx::QueryBuilder,
};

/// Set your mode preference.
///
/// This command will associate the mode you specify with your Discord `UserID` for later use. \
/// This is very helpful on commands such as `/wr` or `/profile` where the bot only wants to fetch \
/// information for a specific mode.
#[poise::command(slash_command, on_error = "Error::handle_command")]
pub async fn mode(
	ctx: Context<'_>,
	#[description = "KZT/SKZ/VNL"] mode: DBModeChoice,
) -> Result<()> {
	trace!("[/mode ({})]", ctx.author().tag());
	trace!("> `mode`: {mode:?}");
	ctx.defer().await?;

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

	let updated = match ctx.find_user_by_id(id).await {
		// User already has a database entry => modify current one
		Ok(user) => {
			// :tf:
			if user.mode.as_ref() == mode.as_ref().ok() {
				ctx.say("You already have this mode set.")
					.await?;
				return Ok(());
			}

			let mut query = QueryBuilder::new(format!(r#"UPDATE {table} SET mode = "#));

			query
				.push_bind(mode_id)
				.push(" WHERE discord_id = ")
				.push_bind(id);

			query
				.build()
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

				let mut query = QueryBuilder::new(format!(
					r#"
					INSERT INTO {table}
					    (name, discord_id, mode)
					"#
				));

				query.push_values([(name, id, mode_id)], |mut query, (name, id, mode_id)| {
					query
						.push_bind(name)
						.push_bind(id)
						.push_bind(mode_id);
				});

				query
					.build()
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
