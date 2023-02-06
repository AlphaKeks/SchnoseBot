//! Discord Bot for CS:GO KZ.
//!
//! You can use this bot to communicate with the [GlobalAPI](https://portal.global-api.com/dashboard) in a convenient way.
//! For example checking world records, personal bests or looking up detailed information about
//! maps. The Bot also uses [KZ:GO](https://kzgo.eu/) and it's API for some extra info.

#![warn(missing_debug_implementations, missing_docs, rust_2018_idioms)]
#![warn(clippy::style, clippy::perf, clippy::complexity, clippy::correctness)]
mod commands;
mod error;
mod global_maps;
use global_maps::GlobalMap;

use {
	clap::{Parser, ValueEnum},
	color_eyre::Result as Eyre,
	gokz_rs::prelude::*,
	log::{debug, info},
	once_cell::sync::OnceCell,
	poise::{
		serenity_prelude::{GatewayIntents, GuildId, UserId},
		Framework, FrameworkOptions, PrefixFrameworkOptions,
	},
	serde::Deserialize,
	sqlx::{mysql::MySqlPoolOptions, MySql, Pool},
	std::{collections::HashSet, path::PathBuf},
};

/// Convenience trait to get data out of a static `OnceCell` so that I can retrieve data with
/// `Result`s instead of `Option`s.
pub trait GlobalMapsContainer {
	/// Tries to get the `OnceCell` but returns a `Result` instead of `Option`. This makes error
	/// handling a lot more convenient.
	fn try_get(&self) -> Result<&Vec<GlobalMap>, error::Error>;

	/// Tries to find a specific map in the `OnceCell`.
	fn find(&self, map_identifier: &MapIdentifier) -> Result<GlobalMap, error::Error>;

	/// Tries to find a specific map name in the `OnceCell`.
	fn find_name(&self, map_name: &str) -> Result<String, error::Error>;
}

/// Cache of all global maps so I don't need to fetch them every time.
static GLOBAL_MAPS: OnceCell<Vec<GlobalMap>> = OnceCell::new();

impl GlobalMapsContainer for OnceCell<Vec<GlobalMap>> {
	fn try_get(&self) -> Result<&Vec<GlobalMap>, error::Error> {
		self.get()
			.ok_or(error::Error::MapNotGlobal)
	}

	fn find(&self, map_identifier: &MapIdentifier) -> Result<GlobalMap, error::Error> {
		self.try_get()?
			.iter()
			.find_map(|map| {
				if let MapIdentifier::ID(map_id) = map_identifier {
					if map.id == *map_id as u16 {
						return Some(map.to_owned());
					}
				}

				if let MapIdentifier::Name(map_name) = map_identifier {
					if map
						.name
						.contains(&map_name.to_lowercase())
					{
						return Some(map.to_owned());
					}
				}

				None
			})
			.ok_or(error::Error::MapNotGlobal)
	}

	fn find_name(&self, map_name: &str) -> Result<String, error::Error> {
		self.find(&MapIdentifier::Name(map_name.to_owned()))
			.map(|map| map.name)
	}
}

#[tokio::main]
async fn main() -> Eyre<()> {
	color_eyre::install()?;
	let args = Args::parse();

	let config_file = std::fs::read_to_string(args.config)?;
	let config: Config = toml::from_str(&config_file)?;

	std::env::set_var(
		"RUST_LOG",
		if args.debug {
			String::from("DEBUG")
		} else if let Some(level) = &config.log_level {
			String::from(level)
		} else {
			String::from("INFO")
		},
	);
	env_logger::init();

	let state = GlobalState::new(config).await;

	let global_maps = global_maps::init(&state.gokz_client).await?;

	GLOBAL_MAPS
		.set(global_maps)
		.expect("This is the first use of the Cell.");

	let framework = Framework::builder()
		.options(FrameworkOptions {
			owners: HashSet::from_iter([UserId(state.config.owner_id)]),
			prefix_options: PrefixFrameworkOptions {
				prefix: Some(String::from("~")),
				ignore_bots: true,
				..Default::default()
			},
			commands: vec![commands::ping(), commands::map()],
			event_handler: |_ctx, event, _framework, _global_state| {
				Box::pin(async {
					debug!("Received event `{}`", event.name());

					Ok(())
				})
			},
			..Default::default()
		})
		.token(&state.config.discord_token)
		.intents(
			GatewayIntents::GUILDS
				| GatewayIntents::GUILD_MEMBERS
				| GatewayIntents::GUILD_MESSAGES
				| GatewayIntents::MESSAGE_CONTENT,
		)
		.setup(move |ctx, _ready, framework| {
			Box::pin(async move {
				let commands = &framework.options().commands;
				match &state.config.mode {
					Mode::Dev => {
						let dev_guild = GuildId(state.config.dev_guild);
						poise::builtins::register_in_guild(ctx, commands, dev_guild).await?;
					}
					Mode::Prod => {
						poise::builtins::register_globally(ctx, commands).await?;
					}
				}

				for command in commands {
					info!(
						"[{}] Successfully registered command `/{}`.",
						&state.config.mode, &command.name
					);
				}

				Ok(state)
			})
		});

	info!("Finished setting up. Connecting to Discord...");
	framework
		.run()
		.await
		.expect("Failed to run framework.");

	Ok(())
}

/// Some convenience CLI arguments to configure the bot quickly without changing the config file.
/// Any of these options will _override_ the values set in the config file.
#[derive(Debug, Clone, Parser)]
struct Args {
	/// The path to the bot's config file.
	#[arg(short, long)]
	#[clap(default_value = "./config.toml")]
	pub config: PathBuf,

	/// Which level to register commands on.
	/// - `Dev`: commands will be registered on a single guild only. This is fast and useful
	///          for development.
	/// - `Prod`: commands will be registered on every guild the bot is on and allowed to register
	///           commands on. This might take a while to reload and therefore should only be used
	///           when running in production.
	#[arg(long)]
	#[clap(default_value = "dev")]
	pub mode: Mode,

	/// Run in debug mode.
	#[arg(long)]
	#[clap(default_value = "false")]
	pub debug: bool,
}

/// Config file used for storing potentially sensitive, as well as non-sensitive but necessary
/// configuration parameters which are needed for the bot to run.
#[derive(Debug, Deserialize)]
pub struct Config {
	/// Can be one of the following:
	/// - `TRACE`
	/// - `DEBUG`
	/// - `INFO`
	/// - `WARN`
	/// - `ERROR`
	///
	/// This value will default to `INFO`.
	/// The `--debug` flag will always override this value to `DEBUG`.
	pub log_level: Option<String>,

	/// Discord API Token for authentication.
	pub discord_token: String,

	/// Steam WebAPI Token for authentication.
	pub steam_token: String,

	/// Which level to register commands on.
	/// - `Dev`: commands will be registered on a single guild only. This is fast and useful
	///          for development.
	/// - `Prod`: commands will be registered on every guild the bot is on and allowed to register
	///           commands on. This might take a while to reload and therefore should only be used
	///           when running in production.
	pub mode: Mode,

	/// The GuildID of the development server. This will be used for registering commands when
	/// running in `Dev` mode.
	pub dev_guild: u64,

	/// The UserID of the bot's owner. This is used for some restricted commands which should only
	/// be used by the bot's owner.
	pub owner_id: u64,

	/// The ChannelID to send report messages to. The bot has a `/report` command which will send
	/// those reports to the `report_channel` channel.
	pub report_channel: u64,

	/// MySQL connection string. The database is used for storing user data.
	pub mysql_url: String,

	/// MySQL table name for storing user data.
	pub mysql_table: String,
}

/// Which level to register commands on.
#[derive(Debug, Clone, Deserialize, ValueEnum)]
pub enum Mode {
	/// Commands will be registered on a single guild only. This is fast and useful for development.
	Dev,

	/// Commands will be registered on every guild the bot is on and allowed to register commands
	/// on. This might take a while to reload and therefore should only be used when running in
	/// production.
	Prod,
}

impl std::fmt::Display for Mode {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}",
			match self {
				Self::Dev => "Dev",
				Self::Prod => "Prod",
			}
		)
	}
}

/// Global State Object used across the entire program. This holds long-living data which I don't
/// want to compute over and over again.
#[derive(Debug)]
pub struct GlobalState {
	/// Config for the bot.
	pub config: Config,

	/// MySQL connection pool for user data.
	pub database: Pool<MySql>,

	/// HTTP Client for making requests with `gokz_rs`.
	pub gokz_client: gokz_rs::Client,
}

impl GlobalState {
	async fn new(config: Config) -> Self {
		let database = MySqlPoolOptions::new()
			.min_connections(10)
			.max_connections(100)
			.connect(&config.mysql_url)
			.await
			.expect("Failed to establish database connection.");

		Self {
			config,
			database,
			gokz_client: gokz_rs::Client::new(),
		}
	}
}

/// Global `Context` type which gets passed to events, commands, etc.
pub type Context<'ctx> = poise::Context<'ctx, GlobalState, crate::error::Error>;

/// Convenience trait to access the Global State more easily. Should be self explanatory.
#[allow(missing_docs)]
pub trait State {
	fn config(&self) -> &Config;
	fn database(&self) -> &Pool<MySql>;
	fn gokz_client(&self) -> &gokz_rs::Client;
}

impl State for Context<'_> {
	fn config(&self) -> &Config {
		&self.data().config
	}

	fn database(&self) -> &Pool<MySql> {
		&self.data().database
	}

	fn gokz_client(&self) -> &gokz_rs::Client {
		&self.data().gokz_client
	}
}