use {
	crate::GlobalStateAccess,
	super::{
		handle_err,
		recompile::{clean, build},
		pull::git_pull,
	},
	crate::SchnoseError,
};

/// Update the bot's code and recompile it
#[poise::command(prefix_command, on_error = "handle_err", owners_only)]
pub async fn update(ctx: crate::Context<'_>) -> Result<(), SchnoseError> {
	ctx.defer().await?;

	let config = &ctx.config();

	let handle = ctx.say("Pulling from GitHub...").await?;
	let mut message = handle.message().await?;
	let message = message.to_mut();

	let old_msg = message.content.clone();
	let git_msg = git_pull(config.git_dir);

	message
		.edit(ctx, |reply| {
			reply.content(format!("{}\n{}\nCleaning build directory...", old_msg, git_msg))
		})
		.await?;

	let old_msg = message.content.clone();
	let clean_msg = clean(config.build_dir);

	message
		.edit(ctx, |reply| {
			reply.content(format!("{}\n{}\nStarting to compile...", old_msg, clean_msg))
		})
		.await?;

	let old_msg = message.content.clone();
	let build_msg = build(config.build_dir, config.build_job_count);

	message
		.edit(ctx, |reply| reply.content(format!("{}\n{}", old_msg, build_msg)))
		.await?;

	Ok(())
}
