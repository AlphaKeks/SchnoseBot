use crate::{error::Error, Context, State};

#[poise::command(
	prefix_command,
	on_error = "Error::handle_command",
	owners_only,
	global_cooldown = 120
)]
pub async fn restart(ctx: Context<'_>) -> Result<(), Error> {
	ctx.defer().await?;

	let msg_handle = ctx.say("Restarting bot...").await?;

	if let Err(why) = crate::process::restart(ctx.config()) {
		let old_content = &msg_handle.message().await?.content;

		msg_handle
			.edit(ctx, |msg| {
				msg.content(format!(
					r#"
{old_content}

Failed to restart:
```
{why}
```"#
				))
			})
			.await?;
	}

	Ok(())
}
