use {
	super::{handle_err, Target},
	crate::{database, GlobalStateAccess, SchnoseError},
	gokz_rs::prelude::*,
	log::trace,
};

/// Check you current database entries.
#[poise::command(slash_command, on_error = "handle_err")]
pub async fn db(
	ctx: crate::Context<'_>,
	#[description = "Do you want others to see the bot's response to this command?"] public: Option<
		bool,
	>,
) -> Result<(), SchnoseError> {
	if public == Some(true) {
		ctx.defer().await?;
	} else {
		ctx.defer_ephemeral().await?;
	}

	trace!("[/db] user: `{}`", &ctx.author().name);

	let database::UserSchema { name, discord_id, steam_id, mode } =
		Target::None(*ctx.author().id.as_u64())
			.query_db(ctx.database(), &format!("discord_id = \"{}\"", ctx.author().id.as_u64()))
			.await?;

	let steam_id = steam_id.unwrap_or_else(|| String::from("NULL"));
	let mode = match mode {
		Some(mode_id) => Mode::try_from(mode_id)?.to_string(),
		None => String::from("NULL"),
	};

	ctx.send(|reply| {
		reply.embed(|e| {
			e.color((116, 128, 194))
				.title(format!("{name}'s database entries"))
				.description(format!(
					"
> player_name: `{name}`
> discord_id: `{discord_id}`
> steam_id: `{steam_id}`
> mode: `{mode}`
				"
				))
		})
	})
	.await?;

	Ok(())
}
