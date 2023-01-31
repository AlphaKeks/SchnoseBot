use {
	super::handle_err,
	crate::GlobalStateAccess,
	crate::SchnoseError,
	log::{error, info},
};

/// Update the bot's code
#[poise::command(prefix_command, on_error = "handle_err", owners_only)]
pub async fn pull(ctx: crate::Context<'_>) -> Result<(), SchnoseError> {
	ctx.defer().await?;

	let config = &ctx.config();

	let handle = ctx
		.say("Pulling from GitHub...")
		.await?;
	let mut message = handle.message().await?;
	let message = message.to_mut();

	let old_msg = message.content.clone();
	let pull_msg = git_pull(config.git_dir);

	message
		.edit(ctx, |reply| reply.content(format!("{old_msg}\n{pull_msg}")))
		.await?;

	Ok(())
}

pub(super) fn git_pull(git_dir: &str) -> String {
	match std::process::Command::new("git")
		.current_dir(git_dir)
		.arg("pull")
		.output()
	{
		Err(why) => {
			let msg = String::from("Failed to pull from GitHub.");
			error!("{}: {:?}", &msg, why);
			msg
		}
		Ok(output) => {
			info!("stdout: {:?}", &output);

			let msg = match (output.stdout.is_empty(), output.stderr.is_empty()) {
				(false, false) => String::from("Pulled new updates."),
				(false, true) => {
					String::from_utf8(output.stdout).expect("stdout should be valid utf-8.")
				}
				_ => String::from_utf8(output.stderr).expect("stderr should be valid utf-8."),
			};

			String::from(msg.trim())
		}
	}
}
