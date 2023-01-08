use {
	crate::GlobalStateAccess,
	super::handle_err,
	crate::SchnoseError,
	log::{info, error},
};

/// recompile the bot binary
#[poise::command(
	prefix_command,
	on_error = "handle_err",
	owners_only,
	global_cooldown = 120
)]
pub async fn recompile(ctx: crate::Context<'_>) -> Result<(), SchnoseError> {
	ctx.defer().await?;

	let config = &ctx.config();

	let handle = ctx.say("Cleaning build directory...").await?;
	let mut message = handle.message().await?;
	let message = message.to_mut();

	let old_msg = message.content.clone();
	let clean_msg = clean(config.build_dir);

	message
		.edit(ctx, |reply| {
			reply.content(format!("{}\n{}\nStarting compilation...", old_msg, clean_msg))
		})
		.await?;

	let old_msg = message.content.clone();
	let build_msg = build(config.build_dir, config.build_job_count);

	message
		.edit(ctx, |reply| reply.content(format!("{}\n{}", old_msg, build_msg)))
		.await?;

	Ok(())
}

pub(super) fn clean(build_dir: &str) -> String {
	match std::process::Command::new("cargo").current_dir(build_dir).arg("clean").output() {
		Err(why) => {
			let msg = String::from("Failed to clean build directory.");
			error!("{}: {:?}", &msg, why);
			msg
		},
		Ok(output) => {
			info!("stdout: {:?}", &output);
			String::from("Finished cleaning build directory.")
		},
	}
}

pub(super) fn build(build_dir: &str, jobs: &str) -> String {
	match std::process::Command::new("cargo")
		.current_dir(build_dir)
		.args(["build", "--release", "--jobs", jobs])
		.output()
	{
		Err(why) => {
			let msg = String::from("Compilation failed.");
			error!("{}: {:?}", &msg, why);
			msg
		},
		Ok(output) => {
			info!("stdout: {:?}", &output);
			String::from("Compilation finished.")
		},
	}
}
