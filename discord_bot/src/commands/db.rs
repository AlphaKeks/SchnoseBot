use {
	super::choices::BoolChoice,
	crate::{db::User, error::Error, Context, State},
	log::trace,
};

/// Check your database entries.
#[poise::command(slash_command, on_error = "Error::handle_command")]
pub async fn db(ctx: Context<'_>, public: Option<BoolChoice>) -> Result<(), Error> {
	trace!("[/db ({})]", ctx.author().tag());
	trace!("> `public`: {public:?}");

	if matches!(public, Some(BoolChoice::Yes)) {
		ctx.defer().await?;
	} else {
		ctx.defer_ephemeral().await?;
	}

	let User { name, discord_id, steam_id, mode } = ctx
		.find_by_id(*ctx.author().id.as_u64())
		.await?;

	let steam_id = steam_id
		.map(|steam_id| steam_id.to_string())
		.unwrap_or_else(|| String::from("NULL"));
	let mode = mode
		.map(|mode| mode.to_string())
		.unwrap_or_else(|| String::from("NULL"));

	ctx.send(|reply| {
		reply.embed(|e| {
			e.color(ctx.color())
				.title(format!("{name}'s Database entries:"))
				.description(format!(
					r#"
> player_name: `{name}`
> discord_id: `{discord_id}`
> steam_id: `{steam_id}`
> mode: `{mode}`
                    "#
				))
				.footer(|f| {
					f.text(ctx.schnose())
						.icon_url(ctx.icon())
				})
		})
	})
	.await?;

	Ok(())
}
