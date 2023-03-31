use {
	clap::Parser, color_eyre::Result as Eyre, gsi_client::gsi, serde::Deserialize,
	std::path::PathBuf, std::sync::mpsc, tracing::error, tracing::Level,
	tracing_subscriber::fmt::format::FmtSpan,
};

mod gui;

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
}

fn main() -> Eyre<()> {
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

	let (tx, rx) = mpsc::channel::<gsi::Info>();

	let state = gui::State::new(config, tx, rx);

	if let Err(why) = state.run(eframe::NativeOptions {
		decorated: true,
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
