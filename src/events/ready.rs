use std::env;

use serenity::model::application::command::Command;
use serenity::model::gateway::{Activity, Ready};
use serenity::{model::prelude::GuildId, prelude::Context};

pub async fn ready(ctx: Context, ready: Ready) {
	let dev_guild = GuildId(
		env::var("DEV_GUILD")
			.expect("no `DEV_GUILD` environment variable found")
			.parse()
			.expect("`DEV_GUILD` environment variable must be an integer"),
	);

	match env::var("MODE") {
		Ok(var) => match var.as_str() {
			"DEV" => {
				let commands =
					GuildId::set_application_commands(&dev_guild, &ctx.http, |commands| {
						commands
							.create_application_command(|cmd| crate::commands::ping::register(cmd))
							.create_application_command(|cmd| {
								crate::commands::setsteam::register(cmd)
							})
							.create_application_command(|cmd| crate::commands::mode::register(cmd))
					})
					.await;

				let mut command_names = vec![];
				if let Ok(command_list) = commands {
					for command in command_list {
						command_names.push(command.name);
					}
				}

				println!(
					"[{}] registered commands: \n> {}",
					var,
					command_names.join("\n> ")
				);
			}

			"PROD" => {
				let commands = Command::set_global_application_commands(&ctx.http, |commands| {
					commands
						.create_application_command(|cmd| crate::commands::ping::register(cmd))
						.create_application_command(|cmd| crate::commands::invite::register(cmd))
						.create_application_command(|cmd| crate::commands::setsteam::register(cmd))
						.create_application_command(|cmd| crate::commands::mode::register(cmd))
						.create_application_command(|cmd| crate::commands::db::register(cmd))
						.create_application_command(|cmd| crate::commands::nocrouch::register(cmd))
						.create_application_command(|cmd| crate::commands::apistatus::register(cmd))
						.create_application_command(|cmd| crate::commands::bpb::register(cmd))
						.create_application_command(|cmd| crate::commands::pb::register(cmd))
						.create_application_command(|cmd| crate::commands::bwr::register(cmd))
						.create_application_command(|cmd| crate::commands::wr::register(cmd))
						.create_application_command(|cmd| crate::commands::recent::register(cmd))
						.create_application_command(|cmd| {
							crate::commands::unfinished::register(cmd)
						})
						.create_application_command(|cmd| crate::commands::random::register(cmd))
						.create_application_command(|cmd| crate::commands::map::register(cmd))
						.create_application_command(|cmd| crate::commands::profile::register(cmd))
				})
				.await;

				println!(
					"[{}] registered commands: \n> {}",
					var,
					(match commands {
						Ok(cmds) => cmds.into_iter().map(|cmd| cmd.name).collect(),
						_ => vec![],
					})
					.join("\n> ")
				);
			}

			_ => panic!(
				"`MODE` environment variable must either be `DEV` or
				`PROD`"
			),
		},
		Err(why) => panic!("{:#?}", why),
	};

	ctx.set_activity(Activity::playing("kz_epiphany_v2")).await;

	println!("connected as {}.", ready.user.tag());
}
