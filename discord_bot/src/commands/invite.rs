use crate::{
	error::{Error, Result},
	Context,
};

/// Invite schnose to your own server!
#[tracing::instrument(skip(ctx), fields(user = ctx.author().tag()))]
#[poise::command(slash_command, ephemeral, on_error = "Error::handle_command")]
pub async fn invite(ctx: Context<'_>) -> Result<()> {
	ctx.say("[click me? 😳](<https://discord.schnose.xyz/>)")
		.await?;
	Ok(())
}
