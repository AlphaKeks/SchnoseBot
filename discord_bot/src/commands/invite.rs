use {
	crate::{
		error::{Error, Result},
		Context,
	},
	log::trace,
};

/// Invite schnose to your own server!
#[poise::command(slash_command, ephemeral, on_error = "Error::handle_command")]
pub async fn invite(ctx: Context<'_>) -> Result<()> {
	trace!("[/invite ({})]", ctx.author().tag());
	ctx.say("[click me? 😳](<https://discord.schnose.xyz/>)")
		.await?;
	Ok(())
}
