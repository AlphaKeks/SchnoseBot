use std::env;

use serenity::model::application::command::Command;
use serenity::model::gateway::Ready;
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
							.create_application_command(|cmd| crate::commands::wr::register(cmd))
							.create_application_command(|cmd| crate::commands::db::register(cmd))
					})
					.await;

				let mut command_names = vec![];
				if let Ok(command_list) = commands {
					for command in command_list {
						command_names.push(command.name);
					}
				}

				println!("[{}] registered commands: {:#?}", var, command_names);
			}

			"PROD" => {
				let commands = Command::create_global_application_command(&ctx.http, |cmd| {
					crate::commands::ping::register(cmd)
				})
				.await;

				println!("[{}] registered commands: {:#?}", var, commands);
			}

			_ => panic!(
				"`MODE` environment variable must either be `DEV` or
				`PROD`"
			),
		},
		Err(why) => panic!("{:#?}", why),
	};

	println!("connected as {}.", ready.user.tag());
}
