use {
	clap::Parser,
	color_eyre::Result as Eyre,
	gsi_client::gsi,
	serde::Deserialize,
	std::path::PathBuf,
	tokio::sync::mpsc,
	tracing::{error, Level},
	tracing_subscriber::fmt::format::FmtSpan,
};

mod gui;
mod http_server;

#[derive(Debug, Parser)]
struct Args {
	/// Path to the configuration file.
	#[arg(short, long = "config")]
	#[clap(default_value = "./config.toml")]
	config_path: PathBuf,

	/// Print debug information.
	#[arg(long)]
	#[clap(default_value = "false")]
	debug: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
	pub cfg_path: String,
	pub port: u16,
	pub api_key: String,
}

#[tokio::main]
async fn main() -> Eyre<()> {
	color_eyre::install()?;
	let args = Args::parse();
	let config_file = std::fs::read_to_string(args.config_path)?;
	let config: Config = toml::from_str(&config_file)?;

	tracing_subscriber::fmt()
		.compact()
		.with_max_level(if args.debug { Level::DEBUG } else { Level::INFO })
		.with_line_number(true)
		.with_span_events(FmtSpan::NEW)
		.init();

	let (tx, rx) = mpsc::unbounded_channel::<gsi::Info>();

	let state = gui::State::new(config, tx, rx);

	if let Err(why) = state.run(eframe::NativeOptions {
		decorated: true,
		drag_and_drop_support: true,
		transparent: true,
		vsync: false,
		hardware_acceleration: eframe::HardwareAcceleration::Preferred,
		default_theme: eframe::Theme::Dark,
		centered: true,
		..Default::default()
	}) {
		error!("Failed to run GUI: {why:#?}");
	}

	Ok(())
}
