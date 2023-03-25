use crate::{
	error::{Error, Result},
	Context,
};

/// Pong!
#[tracing::instrument(skip(ctx), fields(user = ctx.author().tag()))]
#[poise::command(slash_command, ephemeral, on_error = "Error::handle_command")]
pub async fn ping(ctx: Context<'_>) -> Result<()> {
	ctx.say("Pong!").await?;
	Ok(())
}
