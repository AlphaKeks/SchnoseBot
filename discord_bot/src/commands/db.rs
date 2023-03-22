use {
	super::choices::BoolChoice,
	crate::{
		db::User,
		error::{Error, Result},
		Context, State,
	},
	log::trace,
};

/// Check your database entries.
///
/// This command will show you all the information that the bot has saved about your account in \
/// its database. You may specify a `public` option that determines whether other people will be \
/// able to see the bot's response or not.
#[poise::command(slash_command, on_error = "Error::handle_command")]
pub async fn db(
	ctx: Context<'_>,

	#[description = "Send the message so that everybody can see it."]
	#[rename = "public"]
	show_message: Option<BoolChoice>,
) -> Result<()> {
	trace!("[/db ({})]", ctx.author().tag());
	trace!("> `show_message`: {show_message:?}");

	if matches!(show_message, Some(BoolChoice::Yes)) {
		ctx.defer().await?;
	} else {
		ctx.defer_ephemeral().await?;
	}

	let User { name, discord_id, steam_id, mode } = ctx
		.find_user_by_id(*ctx.author().id.as_u64())
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
