use {super::handle_err, crate::SchnoseError};

/// Invite the bot to your server!
#[poise::command(prefix_command, slash_command, on_error = "handle_err", ephemeral)]
pub async fn invite(ctx: crate::Context<'_>) -> Result<(), SchnoseError> {
	ctx.say("[click me? 😳](<https://bot.schnose.eu/>)").await?;
	Ok(())
}
