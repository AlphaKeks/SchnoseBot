#![deny(clippy::perf, clippy::correctness)]
#![warn(
	clippy::style, missing_debug_implementations, rust_2018_idioms, rustdoc::broken_intra_doc_links
)]

#[derive(Debug, Parser)]
struct Args {
	#[arg(short, long = "config")]
	#[clap(default_value = "./config.toml")]
	config_path: PathBuf,

	#[arg(long)]
	#[clap(default_value = "false")]
	debug: bool,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
struct Config {
	mysql_url: String,
}

mod client;
mod commands;
mod db;
mod error;
mod funny_macro;

pub use error::{Error, Result};

use {
	clap::Parser,
	client::GlobalState,
	color_eyre::Result as Eyre,
	serde::Deserialize,
	sqlx::mysql::MySqlPoolOptions,
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
		.with_max_level(if args.debug { Level::DEBUG } else { Level::INFO })
		.with_span_events(FmtSpan::NEW)
		.init();

	let config_file = std::fs::read_to_string(args.config_path)?;
	let config: Config = toml::from_str(&config_file)?;

	let gokz_client = gokz_rs::Client::new();

	let conn_pool = MySqlPoolOptions::new()
		.connect(&config.mysql_url)
		.await?;

	let config = db::get_config(&conn_pool).await?;
	let config = db::update_tokens(config, &gokz_client, &conn_pool).await?;

	let client_config = ClientConfig::new_simple(StaticLoginCredentials {
		credentials: CredentialsPair {
			login: String::from(BOT_NAME),
			token: Some(config.access_token),
		},
	});

	let (mut stream, twitch_client) =
		TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(client_config);

	let global_state =
		GlobalState::new(twitch_client, config.channel_names, gokz_client, conn_pool).await;

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
