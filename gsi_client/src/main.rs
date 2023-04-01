#![windows_subsystem = "windows"]

use {
	clap::Parser,
	color_eyre::Result as Eyre,
	eframe::Storage,
	gsi_client::gsi,
	serde::{Deserialize, Serialize},
	std::{fs::File, io::Write, path::PathBuf},
	tokio::sync::mpsc,
	tracing::{error, info, Level},
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
	pub cfg_path: String,
	pub port: u16,
	pub api_key: String,
}

impl Config {
	pub fn get_config_dir() -> Option<PathBuf> {
		match std::env::consts::OS {
			"linux" | "macos" => {
				let Ok(home_dir) = std::env::var("HOME") else {
					error!("Could not find $HOME");
					return None;
				};

				let mut home_dir = PathBuf::from(home_dir);
				home_dir.push(".config");
				home_dir.push("schnose_gsi");
				home_dir.push("config.toml");
				Some(home_dir)
			}
			"windows" => {
				let Ok(appdata) = std::env::var("APPDATA") else {
					error!("Could not find %APPDATA%.");
					return None;
				};

				let mut appdata = PathBuf::from(appdata);
				appdata.push("schnose_gsi");
				appdata.push("config.toml");
				Some(appdata)
			}
			_ => None,
		}
	}

	pub fn setup(mut config_dir: PathBuf) -> Option<File> {
		let mut file = match File::create(&config_dir) {
			Ok(file) => file,
			Err(why) => {
				if let std::io::ErrorKind::NotFound = why.kind() {
					let full_config_dir = config_dir.clone();
					config_dir.pop();
					match std::fs::create_dir(&config_dir) {
						Ok(()) => match File::create(full_config_dir) {
							Ok(file) => file,
							Err(why) => {
								error!("Failed to create `{config_dir:?}`: {why:#?}");
								return None;
							}
						},
						Err(why) => {
							error!("Failed to create `{config_dir:?}`: {why:#?}");
							return None;
						}
					}
				} else {
					error!("Failed to open `{config_dir:?}`: {why:#?}");
					return None;
				}
			}
		};

		let default_config = r#"cfg_path = ""
port = 3333
api_key = ""
"#;

		if let Err(why) = file.write_all(default_config.as_bytes()) {
			error!("Failed to write out default config: {why:#?}");
		}

		Some(file)
	}
}

impl Storage for Config {
	fn get_string(&self, key: &str) -> Option<String> {
		match key {
			"cfg_path" => Some(self.cfg_path.clone()),
			"api_key" => Some(self.api_key.clone()),
			_ => None,
		}
	}

	fn set_string(&mut self, key: &str, value: String) {
		match key {
			"cfg_path" => self.cfg_path = value,
			"api_key" => self.api_key = value,
			_ => {}
		}
	}

	fn flush(&mut self) {
		let Some(config_dir) = Self::get_config_dir() else {
			error!("Failed to get config dir.");
			return;
		};

		let Some(mut file) = Self::setup(config_dir.clone()) else {
			error!("Failed to create config file.");
			return;
		};

		let Ok(toml_string) = toml::to_string_pretty(self) else {
			error!("Failed to convert config to toml.");
			return;
		};

		if let Err(why) = file.write_all(toml_string.as_bytes()) {
			error!("Failed to save config: {why:#?}");
		}

		info!("Wrote config to `{config_dir:?}`.");
	}
}

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

	let mut config: Config = if args.config_path.exists() {
		let config_file = std::fs::read_to_string(args.config_path)?;
		toml::from_str(&config_file)?
	} else {
		let config_dir = Config::get_config_dir()
			.expect("Missing `config` argument and couldn't find default location.");

		if !config_dir.exists() {
			Config::setup(config_dir.clone());
		}

		let config_file = std::fs::read_to_string(config_dir)?;
		toml::from_str(&config_file)?
	};

	config.flush();

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
