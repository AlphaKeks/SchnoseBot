use {
	std::env,
	crate::commands,
	log::info,
	serenity::{
		prelude::Context,
		model::prelude::{Activity, command::Command, GuildId, Ready},
	},
};

pub(crate) async fn handle(ctx: &Context, ready: &Ready) -> anyhow::Result<()> {
	info!("`READY` event triggered.");
	info!("Connected to Discord as `{}`.", ready.user.tag());

	ctx.set_activity(Activity::playing("kz_epiphany_v2")).await;

	// This variable will determine whether we register commands locally on a dev server or
	// globally on all servers where the bot has permission to register commands.
	let reg_mode = env::var("MODE")?;
	match reg_mode.as_str() {
		"DEV" => register_dev_commands(ctx, &reg_mode).await?,
		"PROD" => register_global_commands(ctx, &reg_mode).await,
		invalid_mode => panic!("`{}` is not a valid mode. Please set the `MODE` environment variable to either `DEV` or `PROD`.", invalid_mode)
	}

	return Ok(());
}

async fn register_dev_commands(ctx: &Context, mode: &str) -> anyhow::Result<()> {
	let dev_guild: u64 = env::var("DEV_GUILD")?.parse()?;
	let dev_guild = GuildId(dev_guild);

	if let Ok(commands) = dev_guild
		.set_application_commands(&ctx.http, |commands| {
			commands
				.create_application_command(|cmd| commands::apistatus::register(cmd))
				.create_application_command(|cmd| commands::bpb::register(cmd))
				.create_application_command(|cmd| commands::bwr::register(cmd))
				.create_application_command(|cmd| commands::db::register(cmd))
				.create_application_command(|cmd| commands::invite::register(cmd))
				// .create_application_command(|cmd| commands::map::register(cmd))
				.create_application_command(|cmd| commands::mode::register(cmd))
				.create_application_command(|cmd| commands::nocrouch::register(cmd))
				.create_application_command(|cmd| commands::pb::register(cmd))
				.create_application_command(|cmd| commands::ping::register(cmd))
				// .create_application_command(|cmd| commands::profile::register(cmd))
				// .create_application_command(|cmd| commands::random::register(cmd))
				.create_application_command(|cmd| commands::recent::register(cmd))
				.create_application_command(|cmd| commands::setsteam::register(cmd))
				// .create_application_command(|cmd| commands::unfinished::register(cmd))
				.create_application_command(|cmd| commands::wr::register(cmd))
		})
		.await
	{
		print_commands(commands, mode);
	}

	return Ok(());
}

async fn register_global_commands(ctx: &Context, mode: &str) {
	if let Ok(commands) = Command::set_global_application_commands(&ctx.http, |commands| {
		commands
			.create_application_command(|cmd| commands::apistatus::register(cmd))
			.create_application_command(|cmd| commands::bpb::register(cmd))
			.create_application_command(|cmd| commands::bwr::register(cmd))
			.create_application_command(|cmd| commands::db::register(cmd))
			.create_application_command(|cmd| commands::invite::register(cmd))
			.create_application_command(|cmd| commands::map::register(cmd))
			.create_application_command(|cmd| commands::mode::register(cmd))
			.create_application_command(|cmd| commands::nocrouch::register(cmd))
			.create_application_command(|cmd| commands::pb::register(cmd))
			.create_application_command(|cmd| commands::ping::register(cmd))
			.create_application_command(|cmd| commands::profile::register(cmd))
			.create_application_command(|cmd| commands::random::register(cmd))
			.create_application_command(|cmd| commands::recent::register(cmd))
			.create_application_command(|cmd| commands::setsteam::register(cmd))
			.create_application_command(|cmd| commands::unfinished::register(cmd))
			.create_application_command(|cmd| commands::wr::register(cmd))
	})
	.await
	{
		print_commands(commands, mode);
	}
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
