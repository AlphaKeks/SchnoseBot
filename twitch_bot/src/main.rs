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
	tokio::time::Instant,
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
		.with_line_number(true)
		.with_span_events(FmtSpan::NEW)
		.init();

	let config_file = std::fs::read_to_string(args.config_path)?;
	let config: Config = toml::from_str(&config_file)?;

	let gokz_client = gokz_rs::Client::new();

	let conn_pool = MySqlPoolOptions::new()
		.connect(&config.mysql_url)
		.await?;

	let config = db::get_config(&conn_pool, !args.debug).await?;
	let config = db::update_tokens(config, &gokz_client, &conn_pool).await?;

	let client_config = ClientConfig::new_simple(StaticLoginCredentials {
		credentials: CredentialsPair {
			login: String::from(BOT_NAME),
			token: Some(config.access_token),
		},
	});

	let (mut stream, twitch_client) =
		TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(client_config);

	let mut global_state =
		GlobalState::new(twitch_client, config.channel_names, gokz_client, conn_pool).await;

	for channel in &global_state.channels {
		info!("Joining `{channel}`");
		global_state
			.client
			.join(channel.to_owned())?;
	}

	let mut last_message = Instant::now();

	while let Some(message) = stream.recv().await {
		match message {
			ServerMessage::Privmsg(mut message) => {
				info!("{}: {:?}", message.sender.name, message.message_text);

				if message.channel_login == "schnosebot" {
					let elapsed = last_message.elapsed().as_secs();

					message.message_text = message
						.message_text
						.chars() // filter out weird unicode characters that 7tv sometimes inserts
						.filter(|c| c.is_ascii() || (c.is_whitespace() && !c.is_ascii_whitespace()))
						.collect();

					match message.message_text.trim() {
						"!join" | "!leave" if elapsed < 30 => {
							let msg = format!(
								"Currently on cooldown. Please wait another {} second(s).",
								30 - elapsed
							);
							global_state
								.send(msg, message, false)
								.await?;
							continue;
						}
						"!join" => {
							global_state
								.join_channel(message)
								.await?;
						}
						"!leave" => {
							global_state
								.leave_channel(message)
								.await?;
						}
						_ => {}
					}

					debug!("Current channels: {:#?}", global_state.channels);

					last_message = Instant::now();
					continue;
				}

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
