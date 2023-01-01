use {super::handle_err, crate::SchnoseError, log::info};

/// Pong!
#[poise::command(prefix_command, slash_command, on_error = "handle_err", ephemeral)]
pub async fn ping(ctx: crate::Context<'_>) -> Result<(), SchnoseError> {
	info!("Pong!");
	ctx.say("Pong!").await?;
	Ok(())
}
