use {
	crate::{
		error::{Error, Result},
		Context,
	},
	log::trace,
};

/// Pong!
#[poise::command(slash_command, ephemeral, on_error = "Error::handle_command")]
pub async fn ping(ctx: Context<'_>) -> Result<()> {
	trace!("[/ping ({})]", ctx.author().tag());
	ctx.say("Pong!").await?;
	Ok(())
}
