use {
	super::handle_err,
	crate::SchnoseError,
	log::{info, error},
};

/// Restart the bot.
#[poise::command(prefix_command, on_error = "handle_err", owners_only)]
pub async fn restart(ctx: crate::Context<'_>) -> Result<(), SchnoseError> {
	ctx.defer().await?;

	let config = &ctx.framework().user_data().await.config;

	let msg = ctx.say("Restarting...").await?;

	let content = &msg.message().await?.content;

	let restart_command = config
		.restart_command
		.split(' ')
		.collect::<Vec<_>>();

	match std::process::Command::new(restart_command[0])
		.current_dir(config.build_dir)
		.args(&restart_command[1..])
		.output()
	{
		Err(why) => {
			error!("Failed to restart: {:?}", why);
			msg.edit(ctx, |reply| reply.content(format!("{}\nFailed to restart.", content)))
				.await?;
		},
		Ok(output) => {
			info!("stdout: {:?}", &output);
		},
	};

	Ok(())
}
