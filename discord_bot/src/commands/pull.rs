use {
	crate::{
		error::{Error, Result},
		Context, State,
	},
	log::trace,
};

/// Pull new changes from GitHub.
#[poise::command(
	prefix_command,
	on_error = "Error::handle_command",
	owners_only,
	global_cooldown = 120
)]
pub async fn pull(ctx: Context<'_>) -> Result<()> {
	trace!("[/pull ({})]", ctx.author().tag());
	ctx.defer().await?;

	let msg_handle = ctx
		.say("Pulling new updates from GitHub...")
		.await?;

	let old_content = &msg_handle.message().await?.content;

	let output = if let Err(why) = crate::process::git_pull(ctx.config()) {
		let old_content = &msg_handle.message().await?.content;
		format!(
			r#"
{old_content}

Failed to pull from GitHub:
```
{why}
```
            "#
		)
	} else {
		format!("{old_content}\nDone.")
	};

	msg_handle
		.edit(ctx, |msg| msg.content(output))
		.await?;

	Ok(())
}