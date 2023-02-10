//! Discord Bot for CS:GO KZ.
//!
//! You can use this bot to communicate with the
//! [GlobalAPI](https://portal.global-api.com/dashboard) in a convenient way. For example checking
//! world records, personal bests or looking up detailed information about maps. The Bot also uses
//! [KZ:GO](https://kzgo.eu/) and it's API for some extra info.

#![warn(missing_debug_implementations, missing_docs, rust_2018_idioms)]
#![warn(clippy::style, clippy::perf, clippy::complexity, clippy::correctness)]

mod commands;
mod custom_types;
mod db;
mod error;
mod global_maps;
mod gokz;
mod process;
mod steam;

use {
	crate::global_maps::GlobalMap,
	clap::{Parser, ValueEnum},
	color_eyre::Result as Eyre,
	gokz_rs::prelude::*,
	log::{debug, info},
	poise::{
		async_trait,
		serenity_prelude::{GatewayIntents, GuildId, UserId},
		Framework, FrameworkOptions, PrefixFrameworkOptions,
	},
	serde::Deserialize,
	sqlx::{mysql::MySqlPoolOptions, MySql, Pool},
	std::{collections::HashSet, path::PathBuf},
};

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

	let framework = Framework::builder()
		.options(FrameworkOptions {
			owners: HashSet::from_iter([UserId(state.config.owner_id)]),
			prefix_options: PrefixFrameworkOptions {
				prefix: Some(String::from("~")),
				ignore_bots: true,
				..Default::default()
			},
			commands: vec![
				commands::apistatus(),
				commands::bmaptop(),
				commands::bpb(),
				commands::btop(),
				commands::bwr(),
				commands::db(),
				commands::help(),
				commands::invite(),
				commands::map(),
				commands::maptop(),
				commands::mode(),
				commands::nocrouch(),
				commands::pb(),
				commands::ping(),
				commands::profile(),
				commands::pull(),
				commands::random(),
				commands::recent(),
				commands::recompile(),
				commands::report(),
				commands::restart(),
				commands::setsteam(),
				commands::top(),
				commands::unfinished(),
				commands::wr(),
			],
			event_handler: |_, event, _, _| {
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
		.setup(move |ctx, _, framework| {
			Box::pin(async move {
				let commands = &framework.options().commands;
				match &state.config.mode {
					RegisterMode::Dev => {
						let dev_guild = GuildId(state.config.dev_guild);
						poise::builtins::register_in_guild(ctx, commands, dev_guild).await?;
					}
					RegisterMode::Prod => {
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
/// Any of these options will override the values set in the config file.
#[derive(Debug, Clone, Parser)]
struct Args {
	/// The path to the bot's config file.
	#[arg(short, long)]
	#[clap(default_value = "./config.toml")]
	pub config: PathBuf,

	/// Which level to register commands on.
	/// - `Dev`: commands will be registered on a single guild only. This is fast and useful for
	///          development.
	/// - `Prod`: commands will be registered on every guild the bot is on and allowed to register
	///           commands on. This might take a while to reload and therefore should only be used
	///           when running in production.
	#[arg(long)]
	#[clap(default_value = "dev")]
	pub mode: RegisterMode,

	/// Run in debug mode.
	#[arg(long)]
	#[clap(default_value = "false")]
	pub debug: bool,
}

/// Config file for the bot.
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

	/// Authentication Token for the Discord API.
	pub discord_token: String,

	/// Authentication Token for the Steam WebAPI.
	pub steam_token: String,

	/// Which level to register commands on.
	/// - `Dev`: commands will be registered on a single guild only. This is fast and useful for
	///          development.
	/// - `Prod`: commands will be registered on every guild the bot is on and allowed to register
	///           commands on. This might take a while to reload and therefore should only be used
	///           when running in production.
	pub mode: RegisterMode,

	/// The [`GuildID`] of the development server. This will be used for registering commands when
	/// running in `Dev` mode.
	pub dev_guild: u64,

	/// The [`UserID`] of the bot's owner. This is used for some restricted commands which should not
	/// be available for everybody to use. (E.g. restarting the bot)
	pub owner_id: u64,

	/// The [`ChannelID`] to send report messages to. The bot has a `/report` command which will
	/// send those reports to the `report_channel` channel.
	pub report_channel: u64,

	/// `MySQL` connection string. The database is used for storing user data.
	pub mysql_url: String,

	/// `MySQL` table name for storing user data.
	pub mysql_table: String,

	/// Shell command to restart the bot's process.
	pub restart_command: String,

	/// Directory in which the bot repository is located.
	pub workspace_directory: String,

	/// Directory for the bot's crate.
	pub bot_directory: String,

	/// How many CPU threads to use for compilation.
	pub jobs: u8,
}

/// Which level to register commands on.
/// - `Dev`: commands will be registered on a single guild only. This is fast and useful for
///          development.
/// - `Prod`: commands will be registered on every guild the bot is on and allowed to register
///           commands on. This might take a while to reload and therefore should only be used
///           when running in production.
#[derive(Debug, Clone, Deserialize, ValueEnum)]
pub enum RegisterMode {
	/// Commands will be registered on a single guild only. This is fast and useful for development.
	Dev,

	/// Commands will be registered on every guild the bot is on and allowed to register commands
	/// on. This might take a while to reload and therefore should only be used when running in
	/// production.
	Prod,
}

impl std::fmt::Display for RegisterMode {
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

/// Global State Object used for the entire runtime of the process. This holds "global" information
/// such as the parsed config file, a database connection pool etc.
#[derive(Debug)]
pub struct GlobalState {
	/// Parsed config file of the bot.
	pub config: Config,

	/// MySQL connection pool for storing user data.
	pub database: Pool<MySql>,

	/// [`gokz_rs::Client`] for making requests with the `gokz_rs` crate.
	pub gokz_client: gokz_rs::Client,

	/// Cache of all global maps.
	pub global_maps: &'static Vec<GlobalMap>,

	/// #7480c2
	pub color: (u8, u8, u8),

	/// (͡ ͡° ͜ つ ͡͡°)
	pub icon: String,

	/// (͡ ͡° ͜ つ ͡͡°)
	pub schnose: String,
}

impl GlobalState {
	async fn new(config: Config) -> Self {
		let database = MySqlPoolOptions::new()
			.min_connections(10)
			.max_connections(100)
			.connect(&config.mysql_url)
			.await
			.expect("Failed to establish database connection.");

		let gokz_client = gokz_rs::Client::new();
		let global_maps = Box::leak(Box::new(
			// global_maps::init(&gokz_client)
			// 	.await
			// 	.expect("Failed to fetch global maps."),
			Vec::new(),
		));

		Self {
			config,
			database,
			gokz_client,
			global_maps,
			color: (116, 128, 194),
			icon: String::from(
				"https://media.discordapp.net/attachments/981130651094900756/1068608508645347408/schnose.png"
			),
			schnose: String::from("(͡ ͡° ͜ つ ͡͡°)")
		}
	}
}

/// Global `Context` type which gets passed to slash commands.
pub type Context<'ctx> = poise::Context<'ctx, GlobalState, crate::error::Error>;

// TODO: It's kind of annoying that I need both of those `Context` types... There might be a way to
// replace it with [`GlobalState`] completeley so I don't need traits to implement convenience
// methods?

/// Global `ApplicationContext` type which gets passed to the `/report` slash commands. It seems to
/// be required to send modals, although I would prefer not having it at all.
pub type ApplicationContext<'ctx> =
	poise::ApplicationContext<'ctx, GlobalState, crate::error::Error>;

/// Convenience trait to access the Global State more easily. Should be self explanatory.
#[allow(missing_docs)]
#[async_trait]
pub trait State {
	fn config(&self) -> &Config;
	fn database(&self) -> &Pool<MySql>;
	fn gokz_client(&self) -> &gokz_rs::Client;
	fn global_maps(&self) -> &'static Vec<GlobalMap>;
	fn get_map(&self, map_identifier: &MapIdentifier) -> Result<GlobalMap, error::Error>;
	fn get_map_name(&self, map_name: &str) -> Result<String, error::Error>;
	fn color(&self) -> (u8, u8, u8);
	fn icon(&self) -> &str;
	fn schnose(&self) -> &str;
	async fn find_by_id(&self, user_id: u64) -> Result<db::User, error::Error>;
	async fn find_by_name(&self, user_name: &str) -> Result<db::User, error::Error>;
	async fn find_by_steam_id(&self, steam_id: &SteamID) -> Result<db::User, error::Error>;
	async fn find_by_mode(&self, mode: Mode) -> Result<db::User, error::Error>;
}

#[rustfmt::skip] // until `fn_single_line` is stable I don't want this to get formatted.
#[async_trait]
impl State for Context<'_> {
	fn config(&self) -> &Config { &self.data().config }
	fn database(&self) -> &Pool<MySql> { &self.data().database }
	fn gokz_client(&self) -> &gokz_rs::Client { &self.data().gokz_client }
	fn global_maps(&self) -> &'static Vec<GlobalMap> { self.data().global_maps }

	fn get_map(&self, map_identifier: &MapIdentifier) -> Result<GlobalMap, error::Error> {
		let mut iter = self.global_maps().iter();
		match map_identifier {
			MapIdentifier::ID(map_id) => iter
				.find_map(|map| if map.id == *map_id as u16 { Some(map.to_owned()) } else { None })
				.ok_or(error::Error::MapNotGlobal),
			MapIdentifier::Name(map_name) => self
				.global_maps()
				.iter()
				.find_map(|map| {
					if map
						.name
						.contains(&map_name.to_lowercase())
					{
						Some(map.to_owned())
					} else {
						None
					}
				})
				.ok_or(error::Error::MapNotGlobal),
		}
	}

	fn get_map_name(&self, map_name: &str) -> Result<String, error::Error> {
		self.global_maps()
			.iter()
			.find_map(|map| {
				if map
					.name
					.contains(&map_name.to_lowercase())
				{
					Some(map.name.clone())
				} else {
					None
				}
			})
			.ok_or(error::Error::MapNotGlobal)
	}

	fn color(&self) -> (u8, u8, u8) { self.data().color }
	fn icon(&self) -> &str { &self.data().icon }
	fn schnose(&self) -> &str { &self.data().schnose }

	async fn find_by_id(&self, user_id: u64) -> Result<db::User, error::Error> {
		Ok(sqlx::query_as::<_, db::UserSchema>(&format!(
			"SELECT * FROM {} WHERE discord_id = {}",
			&self.config().mysql_table,
			user_id,
		))
		.fetch_one(self.database())
		.await?
		.into())
	}

	async fn find_by_name(&self, user_name: &str) -> Result<db::User, error::Error> {
		Ok(sqlx::query_as::<_, db::UserSchema>(&format!(
			r#"SELECT * FROM {} WHERE user_name = "{}""#,
			&self.config().mysql_table,
			user_name
		))
		.fetch_one(self.database())
		.await?
		.into())
	}

	async fn find_by_steam_id(&self, steam_id: &SteamID) -> Result<db::User, error::Error> {
		Ok(sqlx::query_as::<_, db::UserSchema>(&format!(
			r#"SELECT * FROM {} WHERE steam_id = "{}""#,
			&self.config().mysql_table,
			steam_id
		))
		.fetch_one(self.database())
		.await?
		.into())
	}

	async fn find_by_mode(&self, mode: Mode) -> Result<db::User, error::Error> {
		Ok(sqlx::query_as::<_, db::UserSchema>(&format!(
			r#"SELECT * FROM {} WHERE mode = "{}""#,
			&self.config().mysql_table,
			mode as u8
		))
		.fetch_one(self.database())
		.await?
		.into())
	}
}

#[rustfmt::skip] // until `fn_single_line` is stable I don't want this to get formatted.
#[async_trait]
impl State for ApplicationContext<'_> {
	fn config(&self) -> &Config { &self.data().config }
	fn database(&self) -> &Pool<MySql> { &self.data().database }
	fn gokz_client(&self) -> &gokz_rs::Client { &self.data().gokz_client }
	fn global_maps(&self) -> &'static Vec<GlobalMap> { self.data().global_maps }

	fn get_map(&self, map_identifier: &MapIdentifier) -> Result<GlobalMap, error::Error> {
		let mut iter = self.global_maps().iter();
		match map_identifier {
			MapIdentifier::ID(map_id) => iter
				.find_map(|map| if map.id == *map_id as u16 { Some(map.to_owned()) } else { None })
				.ok_or(error::Error::MapNotGlobal),
			MapIdentifier::Name(map_name) => self
				.global_maps()
				.iter()
				.find_map(|map| {
					if map
						.name
						.contains(&map_name.to_lowercase())
					{
						Some(map.to_owned())
					} else {
						None
					}
				})
				.ok_or(error::Error::MapNotGlobal),
		}
	}

	fn get_map_name(&self, map_name: &str) -> Result<String, error::Error> {
		self.global_maps()
			.iter()
			.find_map(|map| {
				if map
					.name
					.contains(&map_name.to_lowercase())
				{
					Some(map.name.clone())
				} else {
					None
				}
			})
			.ok_or(error::Error::MapNotGlobal)
	}

	fn color(&self) -> (u8, u8, u8) { self.data().color }
	fn icon(&self) -> &str { &self.data().icon }
	fn schnose(&self) -> &str { &self.data().schnose }

	async fn find_by_id(&self, user_id: u64) -> Result<db::User, error::Error> {
		Ok(sqlx::query_as::<_, db::UserSchema>(&format!(
			"SELECT * FROM {} WHERE discord_id = {}",
			&self.config().mysql_table,
			user_id,
		))
		.fetch_one(self.database())
		.await?
		.into())
	}

	async fn find_by_name(&self, user_name: &str) -> Result<db::User, error::Error> {
		Ok(sqlx::query_as::<_, db::UserSchema>(&format!(
			r#"SELECT * FROM {} WHERE user_name = "{}""#,
			&self.config().mysql_table,
			user_name
		))
		.fetch_one(self.database())
		.await?
		.into())
	}

	async fn find_by_steam_id(&self, steam_id: &SteamID) -> Result<db::User, error::Error> {
		Ok(sqlx::query_as::<_, db::UserSchema>(&format!(
			r#"SELECT * FROM {} WHERE steam_id = "{}""#,
			&self.config().mysql_table,
			steam_id
		))
		.fetch_one(self.database())
		.await?
		.into())
	}

	async fn find_by_mode(&self, mode: Mode) -> Result<db::User, error::Error> {
		Ok(sqlx::query_as::<_, db::UserSchema>(&format!(
			r#"SELECT * FROM {} WHERE mode = "{}""#,
			&self.config().mysql_table,
			mode as u8
		))
		.fetch_one(self.database())
		.await?
		.into())
	}
}
