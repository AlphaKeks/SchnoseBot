#![windows_subsystem = "windows"]

use {
	clap::Parser,
	color_eyre::Result,
	schnose_gsi_client::{config::Config, gsi::CSGOReport},
	std::path::PathBuf,
	tokio::sync::mpsc,
	tracing::{info, Level},
	tracing_subscriber::fmt::format::FmtSpan,
};

mod gui;

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

	if args.logs || args.debug {
		let log_level = if args.debug { Level::DEBUG } else { Level::INFO };

		tracing_subscriber::fmt()
			.compact()
			.with_max_level(log_level)
			.with_line_number(true)
			.with_span_events(FmtSpan::NEW)
			.init();

		info!("[{log_level}] Initialized logging.");
	}

	// The `sender` will be given to a separate thread that will listen for CS:GO updates and send
	// those upates to the GUI thread using this channel.
	let (gsi_sender, gsi_receiver) = mpsc::unbounded_channel::<CSGOReport>();

	let config = match args.config_path {
		Some(path) => {
			let config_file = std::fs::read_to_string(path)?;
			toml::from_str(&config_file)?
		}
		None => Config::load()?,
	};

	gui::GsiGui::init(gsi_sender, gsi_receiver, config)
		.await
		.expect("Failed to run GUI.");

	Ok(())
}
