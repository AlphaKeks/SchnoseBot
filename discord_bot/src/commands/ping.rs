use crate::{error::Error, Context};

#[poise::command(slash_command, ephemeral, on_error = "Error::handle_command")]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
	ctx.say("Pong!").await?;
	Ok(())
}
