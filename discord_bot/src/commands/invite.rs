use {
	crate::{error::Error, Context},
	log::trace,
};

#[poise::command(slash_command, ephemeral, on_error = "Error::handle_command")]
pub async fn invite(ctx: Context<'_>) -> Result<(), Error> {
	trace!("[/invite] ({:?})", ctx.author());
	ctx.say("[click me? 😳](<https://discord.schnose.xyz/>)")
		.await?;
	Ok(())
}
