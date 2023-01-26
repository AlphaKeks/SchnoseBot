#![warn(missing_debug_implementations, rust_2018_idioms)]

mod commands;
mod database;
mod discord;
mod error;
mod events;
mod formatting;
mod gokz;
mod steam;

use {
	std::collections::HashSet,
	log::info,
	poise::{
		serenity_prelude::{GatewayIntents, GuildId, UserId},
		PrefixFrameworkOptions,
	},
	sqlx::{mysql::MySqlPoolOptions, MySql},
};

/// icon link for footers
const ICON: &str = "https://cdn.discordapp.com/attachments/981130651094900756/981130719537545286/churchOfSchnose.png";

/// Config holding important / sensitive information.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct Config {
	/// Log level
	rust_log: &'static str,
	/// Discord API token to login
	discord_token: &'static str,
	/// Steam WebAPI auth key
	steam_api_key: &'static str,
	/// `DEV` or `PROD`
	pub mode: &'static str,
	/// GuildID fo the dev server
	pub dev_guild_id: u64,
	/// UserID of the bot owner
	pub owner: u64,
	/// ChannelID for sending report messages
	pub report_channel_id: u64,
	/// full path to the git repo
	pub git_dir: &'static str,
	/// full path to the bot's directory
	pub build_dir: &'static str,
	/// number of threads for compilation
	pub build_job_count: &'static str,
	/// shell command to restart the bot's process
	pub restart_command: &'static str,
}

/// the `config.toml` file which will get parsed into [`Config`].
const CONFIG_FILE: &str = include_str!("../config.toml");

/// Global object holding data I don't want to re-compute.
#[derive(Debug)]
pub struct GlobalState {
	pub config: Config,
	pub database: sqlx::Pool<MySql>,
	pub gokz_client: gokz_rs::Client,
}

pub use error::SchnoseError;
pub type Context<'ctx> = poise::Context<'ctx, GlobalState, SchnoseError>;

trait GlobalStateAccess {
	fn config(&self) -> &Config;
	fn database(&self) -> &sqlx::Pool<MySql>;
	fn gokz_client(&self) -> &gokz_rs::Client;
}

impl<'ctx> GlobalStateAccess for Context<'ctx> {
	fn config(&self) -> &Config {
		&self.framework().user_data.config
	}

	fn database(&self) -> &sqlx::Pool<MySql> {
		&self.framework().user_data.database
	}

	fn gokz_client(&self) -> &gokz_rs::Client {
		&self.framework().user_data.gokz_client
	}
}

#[tokio::main]
async fn main() {
	// setup .env
	dotenv::dotenv().expect("Failed to load .env");

	// setup config
	let config: Config = toml::from_str(CONFIG_FILE).expect("Couldn't find `config.toml`.");

	// initialize logging
	std::env::set_var("RUST_LOG", config.rust_log);
	env_logger::init();

	// connect to database
	let database_url = std::env::var("DATABASE_URL").expect("Missing `DATABASE_URL`.");

	let pool = MySqlPoolOptions::new()
		.max_connections(10)
		.connect(&database_url)
		.await
		.expect("Failed to connect to DB.");

	let gokz_client = gokz_rs::Client::new();

	// setup framework
	let framework = poise::Framework::builder()
		.options(poise::FrameworkOptions {
			// Some commands also accept these prefixes, instead of _only_ slash commands.
			prefix_options: PrefixFrameworkOptions {
				prefix: Some(String::from("~")),
				ignore_bots: true,
				..Default::default()
			},
			commands: vec![
				commands::apistatus::apistatus(),
				commands::bmaptop::bmaptop(),
				commands::bpb::bpb(),
				commands::btop::btop(),
				commands::bwr::bwr(),
				commands::db::db(),
				commands::help::help(),
				commands::invite::invite(),
				commands::map::map(),
				commands::maptop::maptop(),
				commands::mode::mode(),
				commands::nocrouch::nocrouch(),
				commands::pb::pb(),
				commands::ping::ping(),
				commands::profile::profile(),
				commands::pull::pull(),
				commands::random::random(),
				commands::report::report(),
				commands::recent::recent(),
				commands::recompile::recompile(),
				commands::restart::restart(),
				commands::setsteam::setsteam(),
				commands::top::top(),
				commands::unfinished::unfinished(),
				commands::update::update(),
				commands::wr::wr(),
			],
			owners: HashSet::from_iter([UserId(config.owner)]),
			event_handler: |ctx, event, framework, global_state| {
				Box::pin(events::handler(ctx, event, framework, global_state))
			},
			..Default::default()
		})
		.token(config.discord_token)
		.intents(
			GatewayIntents::GUILDS
				| GatewayIntents::GUILD_MEMBERS
				| GatewayIntents::GUILD_MESSAGES
				| GatewayIntents::MESSAGE_CONTENT,
		)
		.setup(move |ctx, _, framework| {
			Box::pin(async move {
				let commands = &framework.options().commands;
				match config.mode {
					// `DEV` mode -> register commands on 1 guild only (fast)
					"DEV" => {
						let guild = GuildId(config.dev_guild_id);
						poise::builtins::register_in_guild(ctx, commands, guild).await?;
					},
					// `PROD` mode -> register commands on _every_ guild (slow)
					"PROD" => poise::builtins::register_globally(ctx, commands).await?,
					invalid_mode => {
						panic!("`{}` is an invalid mode. Please use `DEV` or `PROD`.", invalid_mode)
					},
				}
				for command in commands {
					info!("[{}] Successfully registered command `{}`.", config.mode, &command.name);
				}

				Ok(GlobalState { config, database: pool, gokz_client })
			})
		});

	info!("Finished setting up. Connecting to Discord...");
	framework.run().await.unwrap();
}
