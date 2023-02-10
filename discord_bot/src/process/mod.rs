//! Functions to interact with the bot's system process (e.g. restarting).
//!
//! **Do not expose these to random users. Only in `owners_only` commands!**

use {
	crate::{
		error::{Error, Result},
		Config,
	},
	log::{error, info, trace},
	std::process::{Command, Output},
};

pub fn restart(config: &Config) -> Result<()> {
	trace!("[/restart]");

	let restart_command = &config
		.restart_command
		.split(' ')
		.collect::<Vec<&str>>();

	match Command::new(restart_command[0])
		.current_dir(&config.workspace_directory)
		.args(&restart_command[1..])
		.output()
	{
		Ok(Output { .. }) => {
			unreachable!("The bot has restarted... how did we get here?");
		}
		Err(why) => {
			error!("Failed to restart the bot: {why:?}");
			Err(Error::BotRestart)
		}
	}
}

pub fn git_pull(config: &Config) -> Result<(String, String)> {
	trace!("[/pull]");

	match Command::new("git")
		.current_dir(&config.workspace_directory)
		.arg("pull")
		.output()
	{
		Ok(Output { status, stdout, stderr }) => {
			let stdout = String::from_utf8_lossy(&stdout);
			let stderr = String::from_utf8_lossy(&stderr);
			info!("Exit code {status}");
			info!("stdout:\n{}", stdout);
			info!("stderr:\n{}", stderr);
			Ok((stdout.into_owned(), stderr.into_owned()))
		}
		Err(why) => {
			error!("Failed to pull from GitHub: {why:?}");
			Err(Error::GitPull)
		}
	}
}

pub fn cargo_clean(config: &Config) -> Result<(String, String)> {
	match Command::new("cargo")
		.current_dir(&config.workspace_directory)
		.arg("clean")
		.output()
	{
		Ok(Output { status, stdout, stderr }) => {
			let stdout = String::from_utf8_lossy(&stdout);
			let stderr = String::from_utf8_lossy(&stderr);
			info!("Exit code {status}");
			info!("stdout:\n{}", stdout);
			info!("stderr:\n{}", stderr);
			Ok((stdout.into_owned(), stderr.into_owned()))
		}
		Err(why) => {
			error!("Failed to clean target dir: {why:?}");
			Err(Error::CleanTargetDir)
		}
	}
}

pub fn cargo_build(config: &Config) -> Result<(String, String)> {
	trace!("[/recompile]");

	match Command::new("cargo")
		.current_dir(&config.workspace_directory)
		.args([
			"build",
			"--release",
			"--jobs",
			&config.jobs.to_string(),
		])
		.output()
	{
		Ok(Output { status, stdout, stderr }) => {
			let stdout = String::from_utf8_lossy(&stdout);
			let stderr = String::from_utf8_lossy(&stderr);
			info!("Exit code {status}");
			info!("stdout:\n{}", stdout);
			info!("stderr:\n{}", stderr);
			Ok((stdout.into_owned(), stderr.into_owned()))
		}
		Err(why) => {
			error!("Failed to compile: {why:?}");
			Err(Error::Build)
		}
	}
}
