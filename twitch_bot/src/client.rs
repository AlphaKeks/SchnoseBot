use {
	crate::{commands, funny_macro::parse_args, Error, Result},
	color_eyre::{eyre::eyre, Result as Eyre},
	gokz_rs::{MapIdentifier, Mode, PlayerIdentifier},
	schnosebot::global_maps::{self, GlobalMap},
	sqlx::{MySql, Pool, QueryBuilder},
	std::{collections::HashSet, fmt::Display},
	tracing::warn,
	twitch_irc::{
		irc,
		login::StaticLoginCredentials,
		message::PrivmsgMessage,
		transport::tcp::{TCPTransport, TLS},
		TwitchIRCClient,
	},
};

pub type TwitchClient = TwitchIRCClient<TCPTransport<TLS>, StaticLoginCredentials>;

#[derive(Debug)]
pub struct GlobalState {
	pub client: TwitchClient,
	pub channels: HashSet<String>,
	pub gokz_client: gokz_rs::Client,
	pub maps: Vec<GlobalMap>,
	pub conn_pool: Pool<MySql>,
}

impl GlobalState {
	pub async fn new(
		client: TwitchClient,
		channels: Vec<String>,
		gokz_client: gokz_rs::Client,
		conn_pool: Pool<MySql>,
	) -> Self {
		let maps = global_maps::init(&gokz_client)
			.await
			.expect("Failed to fetch global maps.");

		Self {
			client,
			channels: HashSet::from_iter(channels),
			gokz_client,
			maps,
			conn_pool,
		}
	}

	pub fn global_maps(&self) -> &Vec<GlobalMap> {
		&self.maps
	}

	pub fn get_map(&self, map_identifier: impl Into<MapIdentifier>) -> Result<GlobalMap> {
		let map_identifier = map_identifier.into();
		schnosebot::global_maps::fuzzy_find_map(map_identifier.clone(), self.global_maps()).ok_or(
			gokz_rs::Error::InvalidMapIdentifier { value: map_identifier.to_string() }.into(),
		)
	}

	pub async fn send(
		&self,
		message: impl Display,
		ctx: PrivmsgMessage,
		tag_user: bool,
	) -> Eyre<()> {
		let message = if tag_user {
			format!("@{} {}", ctx.sender.name, message)
		} else {
			message.to_string()
		};

		let channel = self
			.channels
			.get(&ctx.channel_login)
			.map(|channel| format!("#{channel}"))
			.ok_or(eyre!("NO CHANNEL FOUND"))?;

		self.client
			.send_message(irc!("PRIVMSG", channel, message.to_string()))
			.await?;

		Ok(())
	}

	pub async fn handle_command(&self, message: PrivmsgMessage) -> Eyre<()> {
		let (reply, tag_user) = match Command::parse(self, message.message_text.clone()) {
			Ok(command) => {
				let tag_user = match command {
					Command::Apistatus => true,
					Command::BPB { .. } => true,
					Command::BWR { .. } => true,
					Command::Map { .. } => true,
					Command::WR { .. } => true,
					Command::PB { .. } => true,
					Command::Player { .. } => true,
					Command::Recent { .. } => true,
					Command::MostRecentRun => true,
				};

				match command.execute(self).await {
					Ok(message) => (message, tag_user),
					Err(why) => (why.to_string(), false),
				}
			}
			Err(why) => (
				match why {
					e @ Error::Unknown => return Err(e.into()),
					Error::Custom(msg) => msg,
					Error::NotACommand => return Ok(()),
					Error::UnknownCommand(_cmd) => {
						// format!("`{cmd}` is not a known command.")
						return Ok(());
					}
					Error::MissingArgs { missing } => format!("Missing arguments: {missing}"),
					Error::IncorrectArgs { expected } => {
						format!("Incorrect arguments. Expected {expected}.")
					}
					Error::GOKZ { message } => message,
					e @ Error::Database => e.to_string(),
					e @ Error::Twitch => e.to_string(),
				},
				true,
			),
		};

		self.send(reply, message, tag_user)
			.await
	}

	pub async fn join_channel(&mut self, ctx: PrivmsgMessage) -> Result<()> {
		let channel_name = &ctx.sender.login;
		let mut query = QueryBuilder::new("INSERT INTO channels (channel_name) VALUES (");
		query.push_bind(channel_name).push(")");
		query
			.build()
			.execute(&self.conn_pool)
			.await?;

		self.channels
			.insert(channel_name.to_owned());

		self.client
			.join(channel_name.to_owned())?;

		self.send(format!("Successfully joined {channel_name}."), ctx, true)
			.await?;

		Ok(())
	}

	pub async fn leave_channel(&mut self, ctx: PrivmsgMessage) -> Result<()> {
		let channel_name = &ctx.sender.login;
		let mut query = QueryBuilder::new("DELETE FROM channels WHERE channel_name = ");
		query.push_bind(channel_name);
		query
			.build()
			.execute(&self.conn_pool)
			.await?;

		self.channels.remove(channel_name);

		self.client
			.part(channel_name.to_owned());

		self.send(format!("Successfully left {channel_name}."), ctx, true)
			.await?;

		Ok(())
	}
}

#[derive(Debug, Clone)]
#[allow(clippy::upper_case_acronyms)]
pub enum Command {
	Apistatus,
	BPB {
		map: GlobalMap,
		player: PlayerIdentifier,
		mode: Mode,
		course: u8,
	},
	BWR {
		map: GlobalMap,
		mode: Mode,
		course: u8,
	},
	Map {
		map: GlobalMap,
	},
	WR {
		map: GlobalMap,
		mode: Mode,
	},
	PB {
		map: GlobalMap,
		player: PlayerIdentifier,
		mode: Mode,
	},
	Player {
		player: PlayerIdentifier,
	},
	Recent {
		player: PlayerIdentifier,
	},
	MostRecentRun,
}

impl Command {
	pub fn parse(state: &GlobalState, message: String) -> Result<Self> {
		if !message.starts_with('!') {
			return Err(Error::NotACommand);
		}

		let message = message.trim().to_owned();

		let (_prefix, args) = message
			.split_once('!')
			.expect("We checked for '!' above.");

		let mut args = args.split(' ').collect::<Vec<_>>();

		let (command_name, message) = (
			args[0],
			args.drain(1..)
				.collect::<Vec<_>>()
				.join(" "),
		);

		match command_name {
			"api" | "apistatus" => Ok(Self::Apistatus),
			"bpb" => {
				let (map, mode, course, player) =
					parse_args!(message, MapIdentifier, "opt" Mode, "opt" u8, PlayerIdentifier)?;
				let map = state.get_map(map)?;
				let mode = mode.unwrap_or(Mode::KZTimer);
				let course = course.unwrap_or(1).max(1);

				Ok(Self::BPB { map, player, mode, course })
			}
			"bwr" => {
				let (map, mode, course) =
					parse_args!(message, MapIdentifier, "opt" Mode, "opt" u8)?;
				let map = state.get_map(map)?;
				let mode = mode.unwrap_or(Mode::KZTimer);
				let course = course.unwrap_or(1).max(1);

				Ok(Self::BWR { map, mode, course })
			}
			"m" | "map" => {
				let map = parse_args!(message, MapIdentifier)?;
				let map = state.get_map(map)?;

				Ok(Self::Map { map })
			}
			"wr" => {
				let (map, mode) = parse_args!(message, MapIdentifier, "opt" Mode)?;
				let map = state.get_map(map)?;
				let mode = mode.unwrap_or(Mode::KZTimer);

				Ok(Self::WR { map, mode })
			}
			"pb" => {
				let (map, mode, player) =
					parse_args!(message, MapIdentifier, "opt" Mode, PlayerIdentifier)?;
				let map = state.get_map(map)?;
				let mode = mode.unwrap_or(Mode::KZTimer);

				Ok(Self::PB { map, player, mode })
			}
			"p" | "player" | "profile" => {
				let player = parse_args!(message, PlayerIdentifier)?;

				Ok(Self::Player { player })
			}
			"recent" => {
				let player = parse_args!(message, PlayerIdentifier)?;

				Ok(Self::Recent { player })
			}
			"mostrecentrun" | "mrr" => Ok(Self::MostRecentRun),
			cmd => Err(Error::UnknownCommand(cmd.to_owned())),
		}
	}

	pub async fn execute(self, state: &GlobalState) -> Result<String> {
		match self {
			Self::Apistatus => commands::apistatus::execute(state).await,
			Self::BPB { map, player, mode, course } => {
				commands::bpb::execute(state, map, player, mode, course).await
			}
			Self::BWR { map, mode, course } => {
				commands::bwr::execute(state, map, mode, course).await
			}
			Self::Map { map } => commands::map::execute(map).await,
			Self::WR { map, mode } => commands::wr::execute(state, map, mode).await,
			Self::PB { map, player, mode } => commands::pb::execute(state, map, player, mode).await,
			Self::Player { player } => commands::player::execute(state, player).await,
			Self::Recent { player } => commands::recent::execute(state, player).await,
			Self::MostRecentRun => commands::mrr::execute(state).await,
		}
	}
}
