use {
	crate::{commands, error::NoArgs, global_maps::GlobalMap, Error, Result},
	color_eyre::{eyre::eyre, Result as Eyre},
	fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher},
	gokz_rs::MapIdentifier,
	std::{collections::HashSet, fmt::Display},
	tracing::error,
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

	pub async fn send(&self, message: impl Display, channel: impl AsRef<str>) -> Eyre<()> {
		let channel = self
			.channels
			.get(channel.as_ref())
			.map(|channel| format!("#{channel}"))
			.ok_or(eyre!("NO CHANNEL FOUND"))?;
		self.client
			.send_message(irc!("PRIVMSG", channel, message.to_string()))
			.await?;
		Ok(())
	}

	fn parse_args(message: String) -> (Option<String>, Option<Vec<String>>) {
		let mut command_name = None;
		let mut args = None;
		if let Some((_, message)) = message.split_once('!') {
			let mut parts = message.split(' ');

			command_name = match parts.next() {
				None => return (None, None),
				Some(name) if name.is_empty() => return (None, None),
				Some(name) => Some(name.to_owned()),
			};

			args = match parts
				.filter_map(|arg| if arg.is_empty() { None } else { Some(arg.to_owned()) })
				.collect::<Vec<_>>()
			{
				args if args.is_empty() => None,
				args => Some(args),
			};
		}
		(command_name, args)
	}

	pub async fn handle_command(&self, message: PrivmsgMessage) -> Eyre<()> {
		let (command, args) = Self::parse_args(message.message_text);
		let Some(command) = dbg!(command) else {
			// User only types `!`
			return Ok(());
		};

		let channel = message.channel_login;

		match match command.as_str() {
			"apistatus" => commands::apistatus(self).await,
			"map" => match args {
				Some(args) => commands::map(self, args).await,
				None => Err(NoArgs::Map.into()),
			},
			cmd => Err(Error::UnknownCommand(cmd.to_owned())),
		} {
			Ok(message) => self.send(message, channel).await?,
			Err(message) => {
				error!("{message}");
				self.send(message.to_string(), channel)
					.await?;
			}
		};

		Ok(())
	}
}
