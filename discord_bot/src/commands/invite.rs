use crate::{error::Error, Context};

#[poise::command(slash_command, ephemeral, on_error = "Error::handle_command")]
pub async fn invite(ctx: Context<'_>) -> Result<(), Error> {
	ctx.say("[click me? 😳](<https://discord.schnose.xyz/>)")
		.await?;
	Ok(())
}
