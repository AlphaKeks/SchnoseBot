use {
	crate::{commands, funny_macro::parse_args, global_maps::GlobalMap, Error, Result},
	color_eyre::{eyre::eyre, Result as Eyre},
	fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher},
	gokz_rs::{MapIdentifier, Mode, PlayerIdentifier},
	std::{collections::HashSet, fmt::Display},
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
}

impl GlobalState {
	pub async fn new(
		client: TwitchClient,
		channels: Vec<String>,
		gokz_client: gokz_rs::Client,
	) -> Self {
		let maps = crate::global_maps::init(&gokz_client)
			.await
			.expect("Failed to fetch global maps.");

		Self {
			client,
			channels: HashSet::from_iter(channels),
			gokz_client,
			maps,
		}
	}

	pub fn global_maps(&self) -> &Vec<GlobalMap> {
		&self.maps
	}

	pub fn get_map(&self, map_identifier: impl Into<MapIdentifier>) -> Result<GlobalMap> {
		let map_identifier = map_identifier.into();
		match map_identifier {
			MapIdentifier::ID(map_id) => self
				.global_maps()
				.iter()
				.find_map(|map| if map.id == map_id { Some(map.to_owned()) } else { None })
				.ok_or(gokz_rs::Error::InvalidMapIdentifier { value: map_id.to_string() }.into()),
			MapIdentifier::Name(map_name) => {
				let fzf = SkimMatcherV2::default();
				let map_name = map_name.to_lowercase();
				self.global_maps()
					.iter()
					.filter_map(|map| {
						let score = fzf.fuzzy_match(&map.name, &map_name)?;
						if score > 50 || map_name.is_empty() {
							return Some((score, map.to_owned()));
						}
						None
					})
					.max_by(|(a_score, _), (b_score, _)| a_score.cmp(b_score))
					.map(|(_, map)| map)
					.ok_or(gokz_rs::Error::InvalidMapIdentifier { value: map_name }.into())
			}
		}
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
				},
				true,
			),
		};

		self.send(reply, message, tag_user)
			.await
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
			"apistatus" => Ok(Self::Apistatus),
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
			"map" => {
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
		}
	}
}
