#![deny(clippy::perf, clippy::correctness)]
#![warn(
	clippy::style, missing_debug_implementations, rust_2018_idioms, rustdoc::broken_intra_doc_links
)]

#[derive(Debug, Parser)]
struct Args {
	#[arg(short, long = "config")]
	#[clap(default_value = "./config.toml")]
	config_path: PathBuf,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
struct Config {
	client_id: String,
	client_secret: String,
	access_token: String,
	refresh_token: String,
	channel_names: Vec<String>,
}

mod client;
mod commands;
mod error;
mod funny_macro;
mod global_maps;
mod util;

pub use error::{Error, Result};

use {
	clap::Parser,
	client::GlobalState,
	color_eyre::Result as Eyre,
	serde::Deserialize,
	std::path::PathBuf,
	tracing::{debug, info, warn, Level},
	tracing_subscriber::fmt::format::FmtSpan,
	twitch_irc::{
		login::{CredentialsPair, StaticLoginCredentials},
		message::ServerMessage,
		ClientConfig, SecureTCPTransport, TwitchIRCClient,
	},
};

const BOT_NAME: &str = "schnosebot";

#[tokio::main]
async fn main() -> Eyre<()> {
	color_eyre::install()?;
	let args = Args::parse();

	tracing_subscriber::fmt()
		.compact()
		.with_max_level(Level::INFO)
		.with_span_events(FmtSpan::NEW)
		.init();

	let config_file = std::fs::read_to_string(args.config_path)?;
	let config: Config = toml::from_str(&config_file)?;

	let gokz_client = gokz_rs::Client::new();

	// if gokz_client
	// 	.get("https://id.twitch.tv/oauth2/validate")
	// 	.header("Authorization", format!("OAuth {}", config.access_token))
	// 	.send()
	// 	.await?
	// 	.error_for_status()
	// 	.is_err()
	// {
	// 	let refresh_token = gokz_client
	// 		.post("https://id.twitch.tv/oauth2/token")
	// 		.header("Content-Type", "application/x-www-form-urlencoded")
	// 		.query(&[
	// 			("client_id", config.client_id.as_str()),
	// 			("client_secret", config.client_secret.as_str()),
	// 			("grant_type", "refresh_token"),
	// 			("refresh_token", config.refresh_token.as_str()),
	// 		])
	// 		.send()
	// 		.await?
	// 		.json::<serde_json::Value>()
	// 		.await?;
	//
	// 	dbg!(&refresh_token);
	//
	// 	config.refresh_token = refresh_token
	// 		.get("refresh_token")
	// 		.expect("No refresh_token field found")
	// 		.to_string();
	//
	// 	config.access_token = refresh_token
	// 		.get("access_token")
	// 		.expect("No refresh_token field found")
	// 		.to_string();
	// }

	let client_config = ClientConfig::new_simple(StaticLoginCredentials {
		credentials: CredentialsPair {
			login: String::from(BOT_NAME),
			token: Some(config.access_token),
		},
	});

	let (mut stream, twitch_client) =
		TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(client_config);

	let global_state = GlobalState::new(twitch_client, config.channel_names, gokz_client).await;

	for channel in &global_state.channels {
		info!("Joining `{channel}`");
		global_state
			.client
			.join(channel.to_owned())?;
	}

	while let Some(message) = stream.recv().await {
		match message {
			ServerMessage::Privmsg(message) => {
				info!("{}: {}", message.sender.name, message.message_text);
				if let Err(why) = global_state
					.handle_command(message)
					.await
				{
					warn!("Command failed: {why:?}");
				}
			}
			message => {
				warn!("got some message");
				debug!("{message:?}");
			}
		}
	}

	Ok(())
}