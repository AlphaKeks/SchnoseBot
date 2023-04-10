#![windows_subsystem = "windows"]

use {
	clap::Parser,
	color_eyre::Result,
	config::Config,
	gui::Client,
	std::{path::PathBuf, sync::Arc},
	tracing::Level,
	tracing_subscriber::fmt::format::FmtSpan,
};

mod config;
mod gsi;
mod gui;
mod logger;
mod server;

#[derive(Debug, Parser)]
struct Args {
	/// Print logs to STDOUT instead of the `logs` tab in the GUI.
	#[arg(long = "logs")]
	#[clap(default_value = "false")]
	logs_to_stdout: bool,

	/// Print `DEBUG` level logs.
	#[arg(long)]
	#[clap(default_value = "false")]
	debug: bool,

	/// Custom config file to use instead of the default one.
	#[arg(short, long = "config")]
	config_path: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
	color_eyre::install()?;
	let args = Args::parse();

	let subscriber = tracing_subscriber::fmt()
		.compact()
		.with_file(true)
		.with_line_number(true)
		.with_span_events(FmtSpan::NEW)
		.with_max_level(match args.debug {
			true => Level::DEBUG,
			false => Level::INFO,
		});

	let logger = match args.logs_to_stdout {
		true => {
			subscriber.init();
			None
		}
		false => {
			let (log_sender, log_receiver) = logger::new();
			let logger = Arc::new(log_sender);
			subscriber
				.json()
				.with_max_level(Level::DEBUG) // Always print debug info when not logging to STDOUT
				.with_writer(Arc::clone(&logger))
				.init();
			Some(log_receiver)
		}
	};

	let config = match args.config_path {
		None => Config::load()?,
		Some(config_path) => {
			let config_file = std::fs::read_to_string(config_path)?;
			toml::from_str(&config_file)?
		}
	};

	Client::init(config, logger).await
}
