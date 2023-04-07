use {
	color_eyre::{eyre::eyre, Result},
	serde::{Deserialize, Serialize},
	std::{fs::File, path::PathBuf},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
	pub csgo_cfg_path: PathBuf,
	pub schnose_api_key: String,
	pub gsi_port: u16,
}

impl Config {
	#[tracing::instrument]
	pub fn get_path() -> Result<PathBuf> {
		#[cfg(unix)]
		let mut home_dir = PathBuf::from(std::env::var("HOME")?);

		#[cfg(windows)]
		let mut home_dir = PathBuf::from(std::env::var("USERPROFILE")?);

		#[cfg(unix)]
		home_dir.push(".config");

		#[cfg(windows)]
		{
			home_dir.push("AppData");
			home_dir.push("Roaming");
		}

		let mut config_dir = home_dir;

		if !config_dir.exists() {
			return Err(eyre!("Config directory ({}) does not exist!", config_dir.display()));
		}

		config_dir.push("schnose_gsi_client");
		config_dir.push("config.toml");
		Ok(config_dir)
	}

	#[tracing::instrument]
	pub fn load() -> Result<Self> {
		use std::io::{Read, Write};

		let config_file = Self::get_path()?;

		// Create config folder if it doesn't exist. This will fail if the folder already exists so
		// we can ignore the potential error.
		let mut config_dir = config_file.clone();
		config_dir.pop();
		_ = std::fs::create_dir(config_dir);

		let mut config_file = match File::options()
			.read(true)
			.write(true)
			.open(&config_file)
		{
			Ok(file) => file,
			Err(err) if err.kind() == std::io::ErrorKind::NotFound => File::options()
				.create(true)
				.read(true)
				.write(true)
				.open(config_file)?,
			Err(err) => {
				return Err(eyre!("Error opening config file: {err:?}"));
			}
		};

		let mut config_file_contents = String::new();
		config_file.read_to_string(&mut config_file_contents)?;

		if let Ok(config) = toml::from_str::<Self>(&config_file_contents) {
			return Ok(config);
		}

		let config_text = r#"csgo_cfg_path = ""
schnose_api_key = ""
gsi_port = 8888"#;

		let config = toml::from_str::<Self>(config_text)?;

		config_file.write_all(config_text.as_bytes())?;

		Ok(config)
	}
}
