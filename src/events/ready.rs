use serenity::model::prelude::{GuildId, command::Command};

use crate::commands;

use {
	std::env,
	crate::schnose::BotState,
	serenity::{
		model::prelude::{Ready, Activity},
		prelude::Context,
	},
};

pub(crate) async fn handle(_state: &BotState, ctx: Context, ready: Ready) -> anyhow::Result<()> {
	log::info!("`ready` event triggered.");

	ctx.set_activity(Activity::playing("kz_epiphany_v2")).await;
	log::warn!("Connected to Discord as `{}`.", ready.user.tag());

	let mode = env::var("MODE")?;
	match mode.as_str() {
		// register commands on dev server only
		"DEV" => {
			let dev_guild = GuildId(env::var("DEV_GUILD")?.parse::<u64>()?);
			if let Ok(commands) = dev_guild
				.set_application_commands(&ctx.http, |commands| {
					commands
						.create_application_command(|cmd| commands::ping::register(cmd))
						.create_application_command(|cmd| commands::apistatus::register(cmd))
						// .create_application_command(|cmd| commands::bpb::register(cmd))
						// .create_application_command(|cmd| commands::bwr::register(cmd))
						.create_application_command(|cmd| commands::db::register(cmd))
						.create_application_command(|cmd| commands::invite::register(cmd))
						// .create_application_command(|cmd| commands::map::register(cmd))
						.create_application_command(|cmd| commands::mode::register(cmd))
						.create_application_command(|cmd| commands::nocrouch::register(cmd))
						// .create_application_command(|cmd| commands::pb::register(cmd))
						// .create_application_command(|cmd| commands::profile::register(cmd))
						// .create_application_command(|cmd| commands::random::register(cmd))
						// .create_application_command(|cmd| commands::recent::register(cmd))
						.create_application_command(|cmd| commands::setsteam::register(cmd))
					// .create_application_command(|cmd| commands::unfinished::register(cmd))
					// .create_application_command(|cmd| commands::wr::register(cmd))
				})
				.await
			{
				print_commands(commands, &mode);
			}
		},
		// register commands globally
		"PROD" => {
			if let Ok(commands) = Command::set_global_application_commands(&ctx.http, |commands| {
				commands
					.create_application_command(|cmd| commands::ping::register(cmd))
					.create_application_command(|cmd| commands::apistatus::register(cmd))
					.create_application_command(|cmd| commands::bpb::register(cmd))
					.create_application_command(|cmd| commands::bwr::register(cmd))
					.create_application_command(|cmd| commands::db::register(cmd))
					.create_application_command(|cmd| commands::invite::register(cmd))
					.create_application_command(|cmd| commands::map::register(cmd))
					.create_application_command(|cmd| commands::mode::register(cmd))
					.create_application_command(|cmd| commands::nocrouch::register(cmd))
					.create_application_command(|cmd| commands::pb::register(cmd))
					.create_application_command(|cmd| commands::profile::register(cmd))
					.create_application_command(|cmd| commands::random::register(cmd))
					.create_application_command(|cmd| commands::recent::register(cmd))
					.create_application_command(|cmd| commands::setsteam::register(cmd))
					.create_application_command(|cmd| commands::unfinished::register(cmd))
					.create_application_command(|cmd| commands::wr::register(cmd))
			})
			.await
			{
				print_commands(commands, &mode);
			}
		},
		invalid_mode => panic!(
			"`{}` is an invalid `MODE` environment variable. Please use `DEV` or `PROD`.",
			invalid_mode
		),
	}

	return Ok(());
}

fn print_commands(commands: Vec<Command>, mode: &str) {
	let names: Vec<String> = commands.into_iter().map(|cmd| cmd.name).collect();

	println!(
		"[MODE: {}] {}",
		mode,
		if names.len() > 0 {
			format!("Registered commands:\n> {}", names.join("\n> "))
		} else {
			String::from("No commands registered.")
		}
	)
}
