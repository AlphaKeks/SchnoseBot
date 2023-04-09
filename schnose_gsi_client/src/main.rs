#![windows_subsystem = "windows"]

use {
	clap::Parser,
	color_eyre::Result,
	logger::Logger,
	schnose_gsi_client::config::Config,
	std::path::PathBuf,
	std::sync::Arc,
	tokio::sync::mpsc,
	tracing::{info, Level},
	tracing_subscriber::fmt::format::FmtSpan,
};

mod gui;
mod logger;

#[derive(Debug, Parser)]
struct Args {
	#[arg(long)]
	#[clap(default_value = "false")]
	logs: bool,

	#[arg(long)]
	#[clap(default_value = "false")]
	debug: bool,

	#[arg(short, long)]
	config_path: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
	let args = Args::parse();

	let log_level = if args.debug { Level::DEBUG } else { Level::INFO };

	let (log_sender, log_receiver) = mpsc::unbounded_channel();

	let logger = Arc::new(Logger::new(log_sender));

	tracing_subscriber::fmt()
		.compact()
		.json()
		.with_max_level(log_level)
		.with_file(true)
		.with_line_number(true)
		.with_span_events(FmtSpan::NEW)
		.with_writer(Arc::clone(&logger))
		.init();

	info!("[{log_level}] Initialized logging.");

	let config = match args.config_path {
		Some(path) => {
			let config_file = std::fs::read_to_string(path)?;
			toml::from_str(&config_file)?
		}
		None => Config::load()?,
	};

	gui::GsiGui::init(config, log_receiver)
		.await
		.expect("Failed to run GUI");

	Ok(())
}
