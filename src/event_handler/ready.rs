use crate::commands;

use std::env;

use serenity::{
	model::{
		application::command::Command,
		gateway::Ready,
		prelude::{Activity, GuildId},
	},
	prelude::Context,
};

pub async fn handle(ctx: Context, ready: Ready) {
	// check whether we're running in dev or production
	// mode so we don't spam the Discord API if we don't
	// necessarily have to
	if let Ok(env_mode) = env::var("MODE") {
		match env_mode.as_str() {
			"DEV" => {
				let dev_guild = GuildId(
					env::var("DEV_GUILD")
						.expect("`DEV_GUILD` variable needs to be set to run in `DEV` mode.")
						.parse::<u64>()
						.expect("`DEV_GUILD` variable must be a 64-bit integer."),
				);

				// register commands for a specific
				// guild => good for testing because
				// it's faster than globally registering
				// commands
				let commands =
					GuildId::set_application_commands(&dev_guild, &ctx.http, |commands| {
						commands
							.create_application_command(|cmd| commands::ping::register(cmd))
							.create_application_command(|cmd| commands::invite::register(cmd))
							.create_application_command(|cmd| commands::setsteam::register(cmd))
							.create_application_command(|cmd| commands::mode::register(cmd))
							.create_application_command(|cmd| commands::db::register(cmd))
						// .create_application_command(|cmd| commands::nocrouch::register(cmd))
						// .create_application_command(|cmd| commands::apistatus::register(cmd))
						// .create_application_command(|cmd| commands::bpb::register(cmd))
						// .create_application_command(|cmd| commands::pb::register(cmd))
						// .create_application_command(|cmd| commands::bwr::register(cmd))
						// .create_application_command(|cmd| commands::wr::register(cmd))
						// .create_application_command(|cmd| commands::recent::register(cmd))
						// .create_application_command(|cmd| commands::unfinished::register(cmd))
						// .create_application_command(|cmd| commands::random::register(cmd))
						// .create_application_command(|cmd| commands::map::register(cmd))
						// .create_application_command(|cmd| commands::profile::register(cmd))
					})
					.await;

				let command_names = match commands {
					Err(why) => {
						println!("Failed to collect command names. {:#?}", why);
						return;
					},
					Ok(commands) => {
						commands.into_iter().map(|cmd| cmd.name).collect::<Vec<String>>()
					},
				};

				println!(
					"[{}] registered commands:{}",
					env_mode,
					if command_names.len() > 0 {
						format!("\n> {}", command_names.join("\n> "))
					} else {
						String::new()
					}
				);
			},
			"PROD" => {
				// register commands globally
				// intended for production; slower but
				// makes commands available on all
				// servers that the bot is on
				let commands = Command::set_global_application_commands(&ctx.http, |commands| {
					commands
						.create_application_command(|cmd| commands::ping::register(cmd))
						.create_application_command(|cmd| commands::invite::register(cmd))
						.create_application_command(|cmd| commands::setsteam::register(cmd))
						.create_application_command(|cmd| commands::mode::register(cmd))
						.create_application_command(|cmd| commands::db::register(cmd))
						.create_application_command(|cmd| commands::nocrouch::register(cmd))
						.create_application_command(|cmd| commands::apistatus::register(cmd))
						.create_application_command(|cmd| commands::bpb::register(cmd))
						.create_application_command(|cmd| commands::pb::register(cmd))
						.create_application_command(|cmd| commands::bwr::register(cmd))
						.create_application_command(|cmd| commands::wr::register(cmd))
						.create_application_command(|cmd| commands::recent::register(cmd))
						.create_application_command(|cmd| commands::unfinished::register(cmd))
						.create_application_command(|cmd| commands::random::register(cmd))
						.create_application_command(|cmd| commands::map::register(cmd))
						.create_application_command(|cmd| commands::profile::register(cmd))
				})
				.await;

				let command_names = match commands {
					Err(why) => {
						println!("Failed to collect command names. {:#?}", why);
						return;
					},
					Ok(commands) => {
						commands.into_iter().map(|cmd| cmd.name).collect::<Vec<String>>()
					},
				};

				println!(
					"[{}] registered commands:{}",
					env_mode,
					if command_names.len() > 0 {
						format!("\n> {}", command_names.join("\n> "))
					} else {
						String::new()
					}
				);
			},
			_ => unimplemented!("`MODE` variable can only be \"DEV\" or \"PROD\"."),
		}
	}

	// set activity status
	ctx.set_activity(Activity::playing("kz_epiphany_v2")).await;

	// confirm connection to Discord
	println!("Connected to Discord as {}.", ready.user.tag());
}
