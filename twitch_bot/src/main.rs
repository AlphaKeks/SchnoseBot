#![allow(unused)]

use {
	clap::Parser,
	color_eyre::{eyre::eyre, Result as Eyre},
	gokz_rs::{global_api, schnose_api, MapIdentifier},
	serde::Deserialize,
	std::{collections::HashSet, path::PathBuf},
	tracing::{debug, error, info},
	twitch_irc::{
		irc,
		login::{CredentialsPair, StaticLoginCredentials},
		message::{IRCMessage, ServerMessage},
		transport::tcp::{TCPTransport, TLS},
		ClientConfig, SecureTCPTransport, TwitchIRCClient,
	},
};

#[derive(Debug, Parser)]
struct Args {
	#[arg(short, long = "config")]
	#[clap(default_value = "./config.toml")]
	config_path: PathBuf,
}

#[derive(Debug, Deserialize)]
struct Config {
	oauth_token: String,
	channel_name: String,
}

// macro_rules! sendmsg {
// 	($message:expr, $channel:expr, $client:expr) => {
// 		$client
// 			.send_message(irc!("PRIVMSG", $channel, $message))
// 			.await
// 	};
// }

type TwitchClient = TwitchIRCClient<TCPTransport<TLS>, StaticLoginCredentials>;

#[derive(Debug)]
struct GlobalState {
	client: TwitchClient,
	channels: HashSet<String>,
	gokz_client: gokz_rs::Client,
}

impl std::ops::Deref for GlobalState {
	type Target = TwitchClient;

	fn deref(&self) -> &Self::Target {
		&self.client
	}
}

impl GlobalState {
	pub async fn send(&self, message: impl AsRef<str>, channel: impl AsRef<str>) -> Eyre<()> {
		let channel = self
			.channels
			.get(channel.as_ref())
			.map(|channel| format!("#{channel}"))
			.ok_or(eyre!("NO CHANNEL FOUND"))?;
		self.send_message(irc!("PRIVMSG", channel, message.as_ref()))
			.await?;
		Ok(())
	}
}

#[tokio::main]
async fn main() -> Eyre<()> {
	color_eyre::install()?;
	let args = Args::parse();

	tracing_subscriber::fmt()
		.compact()
		.init();

	let config_file = std::fs::read_to_string(args.config_path)?;
	let config: Config = toml::from_str(&config_file)?;

	let client_config = ClientConfig::new_simple(StaticLoginCredentials {
		credentials: CredentialsPair {
			login: String::from("schnosebot"),
			token: Some(config.oauth_token),
		},
	});

	let (mut incoming_messages, client) =
		TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(client_config);

	client.join(config.channel_name.clone())?;

	let state = GlobalState {
		client,
		channels: HashSet::from_iter([config.channel_name]),
		gokz_client: gokz_rs::Client::new(),
	};

	info!("Joined {:?}.", &state.channels);
	state
		.send("Hello, world!", "alphakekskz")
		.await?;

	while let Some(message) = incoming_messages.recv().await {
		match message {
			ServerMessage::Privmsg(msg) => {
				println!("{}: {}", msg.sender.name, msg.message_text);

				if let Some((_, message)) = msg.message_text.split_once('!') {
					let mut parts = message.split(' ');
					let Some(command_name) = parts.next() else {
						continue;
					};
					let args: Vec<_> = parts
						.filter(|arg| !arg.is_empty())
						.collect();

					if let Err(why) =
						handle_command(command_name, args, &msg.channel_login, &state).await
					{
						error!("Error while handling command: {why}");
						state
							.send(why.to_string(), &msg.channel_login)
							.await?;
					}
				}
			}
			message => debug!("Received message: {:?}", message),
		};
	}

	Ok(())
}

async fn handle_command(
	command: impl AsRef<str>,
	args: Vec<&str>,
	channel: impl AsRef<str>,
	state: &GlobalState,
) -> Eyre<()> {
	match command.as_ref() {
		"map" => {
			let Some(map_ident) = args.first() else {
				return Err(eyre!("missing map identifier"));
			};

			let map_ident = map_ident.parse::<MapIdentifier>()?;

			let map = schnose_api::get_map(&map_ident, &state.gokz_client).await?;

			let message = format!(
				"[T{}] {} - Mapper: {} - Last Update: {} - Bonuses: {}",
				map.tier as u8,
				map.name,
				map.mapper_name,
				map.updated_on,
				map.courses.len() - 1
			);

			state.send(message, channel).await?;
		}
		"wr" => {
			todo!();
		}
		_ => {
			return Err(eyre!("Unknown command"));
		}
	}

	Ok(())
}
