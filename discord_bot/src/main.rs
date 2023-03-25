//! Discord Bot for CS:GO KZ.
//!
//! You can use this bot for running commands you already know from GOKZ, but in Discord!
//! The bot uses a `MySql` database to store user information such as their [`SteamID`] and favorite
//! [`Mode`]. To fetch information about players, maps, etc. it uses the [`gokz_rs`] crate. If you
//! have any suggestions or bug reports, feel free to submit an
//! [issue on GitHub](https://github.com/AlphaKeks/SchnoseBot/issues)!

#![warn(missing_debug_implementations, missing_docs, rust_2018_idioms)]
#![warn(clippy::style, clippy::perf, clippy::complexity, clippy::correctness)]

mod commands;
mod db;
mod error;
mod global_maps;
mod gokz;
mod process;
mod steam;
mod target;

use {
	crate::{
		error::{Error, Result},
		global_maps::GlobalMap,
	},
	clap::{Parser, ValueEnum},
	color_eyre::Result as Eyre,
	fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher},
	gokz_rs::{MapIdentifier, Mode, SteamID},
	poise::{
		async_trait,
		serenity_prelude::{Activity, GatewayIntents, GuildId, UserId},
		Command, Event, Framework, FrameworkOptions, PrefixFrameworkOptions,
	},
	serde::Deserialize,
	sqlx::{mysql::MySqlPoolOptions, MySql, Pool, QueryBuilder},
	std::{collections::HashSet, path::PathBuf},
	time::macros::format_description,
	tracing::{debug, info},
	tracing_subscriber::{
		fmt::{format::FmtSpan, time::UtcTime},
		EnvFilter,
	},
};

#[tokio::main]
async fn main() -> Eyre<()> {
	color_eyre::install()?;
	let args = Args::parse();

	let config_file = std::fs::read_to_string(args.config)?;
	let config: Config = toml::from_str(&config_file)?;

	let cwd = std::env::var("PWD")?;
	let file_logger = tracing_appender::rolling::minutely(cwd + "/logs", "schnosebot.log");
	let (log_writer, _guard) = tracing_appender::non_blocking(file_logger);

	tracing_subscriber::fmt()
		.compact()
		.with_writer(log_writer)
		.with_timer(UtcTime::new(format_description!(
			"[[[year]-[month]-[day] | [hour]:[minute]:[second]]"
		)))
		.with_line_number(true)
		.with_span_events(FmtSpan::NEW)
		.with_env_filter({
			EnvFilter::new(if args.debug {
				"DEBUG"
			} else if let Some(ref level) = config.log_level {
				level.as_str()
			} else {
				"discord_bot=INFO,gokz_rs=INFO"
			})
		})
		.init();

	let global_state = GlobalState::new(config).await;

	let framework = Framework::builder()
		.options(FrameworkOptions {
			owners: HashSet::from_iter([UserId(global_state.config.owner_id)]),
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
			event_handler: |ctx, event, _, _| {
				Box::pin(async move {
					debug!("Received event `{}`", event.name());
					if let Event::Ready { data_about_bot } = event {
						info!("Connected to Discord as {}!", data_about_bot.user.tag());

						// Change status every 5 minutes to a different map name
						let mut old_idx = global_maps::BASED_MAPS.len();
						loop {
							let idx = old_idx % global_maps::BASED_MAPS.len();
							let map = global_maps::BASED_MAPS[idx];
							ctx.set_activity(Activity::playing(map))
								.await;
							old_idx += 1;
							tokio::time::sleep(tokio::time::Duration::from_secs(300)).await;
						}
					}
					Ok(())
				})
			},
			..Default::default()
		})
		.token(&global_state.config.discord_token)
		.intents(
			GatewayIntents::GUILDS
				| GatewayIntents::GUILD_MEMBERS
				| GatewayIntents::GUILD_MESSAGES
				| GatewayIntents::MESSAGE_CONTENT,
		)
		.setup(move |ctx, _, framework| {
			Box::pin(async move {
				let commands = &framework.options().commands;
				let mode = &global_state.config.mode;
				match mode {
					RegisterMode::Dev => {
						let dev_guild = GuildId(global_state.config.dev_guild);
						poise::builtins::register_in_guild(ctx, commands, dev_guild).await?;
					}
					RegisterMode::Prod => {
						poise::builtins::register_globally(ctx, commands).await?;
					}
				}

				for Command { name, .. } in commands {
					info!("[{mode}] Successfully registered command `/{name}`.");
				}

				Ok(global_state)
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
		f.write_str(match self {
			Self::Dev => "Dev",
			Self::Prod => "Prod",
		})
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

	/// Cache of all global map names.
	pub global_map_names: &'static Vec<&'static str>,

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
		let global_maps: &'static Vec<GlobalMap> = Box::leak(Box::new(
			global_maps::init(&gokz_client)
				.await
				.expect("Failed to fetch global maps."),
		));
		let global_map_names: &'static Vec<&'static str> = Box::leak(Box::new(
			global_maps
				.iter()
				.map(|map| map.name.as_str())
				.collect::<Vec<&str>>(),
		));

		Self {
			config,
			database,
			gokz_client,
			global_maps,
			global_map_names,
			color: (116, 128, 194),
			icon: String::from(
				"https://media.discordapp.net/attachments/981130651094900756/1068608508645347408/schnose.png"
			),
			schnose: String::from("(͡ ͡° ͜ つ ͡͡°)")
		}
	}
}

/// Global `Context` type which gets passed to slash commands.
pub type Context<'ctx> = poise::Context<'ctx, GlobalState, Error>;

/// Convenience trait for getter functions on [`Context`] since it's not my own type and I haven't
/// figured out how to replace it yet.
#[allow(missing_docs)]
#[async_trait]
pub trait State {
	fn config(&self) -> &Config;
	fn database(&self) -> &Pool<MySql>;
	fn gokz_client(&self) -> &gokz_rs::Client;
	fn global_maps(&self) -> &'static Vec<GlobalMap>;
	fn global_map_names(&self) -> &'static Vec<&'static str>;
	fn get_map(&self, map_identifier: &MapIdentifier) -> Result<GlobalMap>;

	fn get_map_name(&self, map_name: &str) -> Result<String> {
		self.get_map(&MapIdentifier::Name(map_name.to_owned()))
			.map(|map| map.name)
	}

	fn color(&self) -> (u8, u8, u8);
	fn icon(&self) -> &str;
	fn schnose(&self) -> &str;
	async fn find_user_by_id(&self, user_id: u64) -> Result<db::User>;
	async fn find_user_by_name(&self, user_name: &str) -> Result<db::User>;
	async fn find_user_by_steam_id(&self, steam_id: &SteamID) -> Result<db::User>;
	async fn find_user_by_mode(&self, mode: Mode) -> Result<db::User>;
}

#[async_trait]
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

	fn global_maps(&self) -> &'static Vec<GlobalMap> {
		self.data().global_maps
	}

	fn global_map_names(&self) -> &'static Vec<&'static str> {
		self.data().global_map_names
	}

	fn get_map(&self, map_identifier: &MapIdentifier) -> Result<GlobalMap> {
		match map_identifier {
			MapIdentifier::ID(map_id) => self
				.global_maps()
				.iter()
				.find_map(|map| if map.id == *map_id { Some(map.to_owned()) } else { None })
				.ok_or(Error::MapNotGlobal),
			MapIdentifier::Name(map_name) => {
				let fzf = SkimMatcherV2::default();
				let map_name = map_name.to_lowercase();
				self.global_maps()
					.iter()
					.filter_map(move |map| {
						let score = fzf.fuzzy_match(&map.name, &map_name)?;
						if score > 50 || map_name.is_empty() {
							return Some((score, map.to_owned()));
						}
						None
					})
					.max_by(|(a_score, _), (b_score, _)| a_score.cmp(b_score))
					.map(|(_, map)| map)
					.ok_or(Error::MapNotGlobal)
			}
		}
	}

	fn color(&self) -> (u8, u8, u8) {
		self.data().color
	}
	fn icon(&self) -> &str {
		&self.data().icon
	}

	fn schnose(&self) -> &str {
		&self.data().schnose
	}

	async fn find_user_by_id(&self, user_id: u64) -> Result<db::User> {
		let mut query = QueryBuilder::new(format!(
			r#"SELECT * FROM {} WHERE discord_id = "#,
			self.config().mysql_table
		));

		query.push_bind(user_id);

		Ok(query
			.build_query_as::<db::UserSchema>()
			.fetch_one(self.database())
			.await?
			.into())
	}

	async fn find_user_by_name(&self, user_name: &str) -> Result<db::User> {
		let mut query = QueryBuilder::new(format!(
			r#"SELECT * FROM {} WHERE name LIKE "%"#,
			self.config().mysql_table
		));

		query.push_bind(user_name).push(r#"%""#);

		Ok(query
			.build_query_as::<db::UserSchema>()
			.fetch_one(self.database())
			.await?
			.into())
	}

	async fn find_user_by_steam_id(&self, steam_id: &SteamID) -> Result<db::User> {
		let mut query = QueryBuilder::new(format!(
			r#"SELECT * FROM {} WHERE steam_id = "#,
			self.config().mysql_table
		));

		query.push_bind(steam_id.to_string());

		Ok(query
			.build_query_as::<db::UserSchema>()
			.fetch_one(self.database())
			.await?
			.into())
	}

	async fn find_user_by_mode(&self, mode: Mode) -> Result<db::User> {
		let mut query = QueryBuilder::new(format!(
			r#"SELECT * FROM {} WHERE mode = "#,
			self.config().mysql_table
		));

		query.push_bind(mode as u8);

		Ok(query
			.build_query_as::<db::UserSchema>()
			.fetch_one(self.database())
			.await?
			.into())
	}
}
