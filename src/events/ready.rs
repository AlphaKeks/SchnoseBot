use {
	std::env,
	anyhow::Result,
	crate::{commands, Schnose},
	serenity::{
		prelude::Context,
		model::{
			application::command::Command,
			prelude::{Activity, GuildId, Ready},
		},
	},
};

pub async fn handle(_client: &Schnose, ctx: Context, ready: Ready) -> Result<()> {
	// set status
	ctx.set_activity(Activity::playing("kz_epiphany_v2")).await;

	println!("Connected to Discord as {}.", ready.user.tag());

	// registering commands
	let mode = env::var("MODE")?;
	match mode.as_str() {
		"DEV" => {
			let dev_guild = GuildId(env::var("DEV_GUILD")?.parse::<u64>()?);
			if let Ok(commands) = dev_guild
				.set_application_commands(&ctx.http, |commands| {
					commands
						.create_application_command(|cmd| commands::ping::register(cmd))
						// .create_application_command(|cmd| commands::apistatus::register(cmd))
						// .create_application_command(|cmd| commands::bpb::register(cmd))
						// .create_application_command(|cmd| commands::bwr::register(cmd))
						.create_application_command(|cmd| commands::db::register(cmd))
						.create_application_command(|cmd| commands::invite::register(cmd))
						// .create_application_command(|cmd| commands::map::register(cmd))
						.create_application_command(|cmd| commands::mode::register(cmd))
						// .create_application_command(|cmd| commands::nocrouch::register(cmd))
						// .create_application_command(|cmd| commands::pb::register(cmd))
						// .create_application_command(|cmd| commands::profile::register(cmd))
						// .create_application_command(|cmd| commands::random::register(cmd))
						// .create_application_command(|cmd| commands::recent::register(cmd))
						.create_application_command(|cmd| commands::setsteam::register(cmd))
				})
				.await
			{
				let command_names: Vec<String> = commands.into_iter().map(|cmd| cmd.name).collect();

				println!(
					"[MODE: {}] {}",
					mode,
					if command_names.len() > 0 {
						format!("Registered commands:\n> {}", command_names.join("\n> "))
					} else {
						String::from("No commands registered.")
					}
				);
			}
		},
		"PROD" => {
			if let Ok(commands) = Command::set_global_application_commands(&ctx.http, |_commands| {
				todo!("register global commands here.")
			})
			.await
			{
				let command_names: Vec<String> = commands.into_iter().map(|cmd| cmd.name).collect();

				println!(
					"[MODE: {}] {}",
					mode,
					if command_names.len() > 0 {
						format!("Registered commands:\n> {}", command_names.join("\n> "))
					} else {
						String::from("No commands registered.")
					}
				);
			}
		},
		_ => unreachable!("[env] Invalid `MODE`. Use `DEV` or `PROD`."),
	}

	return Ok(());
}
