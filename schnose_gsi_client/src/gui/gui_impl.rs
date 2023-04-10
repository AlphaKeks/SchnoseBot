use eframe::egui::CentralPanel;

use {
	super::{Client, Tab},
	crate::config::Config,
	eframe::egui::TopBottomPanel,
	std::{fs::File, io::Write},
	tracing::{error, info},
};

impl eframe::App for Client {
	/// This function will be ran every frame.
	fn update(&mut self, ctx: &eframe::egui::Context, _: &mut eframe::Frame) {
		TopBottomPanel::top("heading-panel").show(ctx, |ui| {
			ui.add_space(12.0);

			ui.horizontal(|ui| {
				ui.selectable_value(&mut self.current_tab, Tab::Main, "Main");
				ui.selectable_value(&mut self.current_tab, Tab::Logs, "Logs");
			});

			ui.separator();
			ui.add_space(12.0);
			self.render_status(ui);
			ui.add_space(12.0);
		});

		CentralPanel::default().show(ctx, |ui| {
			ui.add_space(12.0);
			match self.current_tab {
				Tab::Main => self.render_main(ui),
				Tab::Logs => self.render_logs(ui),
			};
		});
	}

	/// This function will be ran periodically and before the application shuts down to save
	/// information in a config file.
	#[tracing::instrument]
	fn save(&mut self, _: &mut dyn eframe::Storage) {
		let Ok(config_path) = Config::get_path() else {
			return error!("Failed to locate config file.");
		};

		let config = tokio::task::block_in_place(|| self.config.blocking_lock().clone());

		let Ok(mut config_file) = File::options().write(true).create(true).open(&config_path) else {
			return error!("Failed to open config file at `{}`.", config.csgo_cfg_path.display());
		};

		let Ok(config_as_toml) = toml::to_string_pretty(&config) else {
			return error!("Failed to serialize config as toml. ({:#?})", self.config);
		};

		if let Err(why) = config_file.write_all(config_as_toml.as_bytes()) {
			return error!("Failed to write to config file: {why:#?}");
		}

		info!("Successfully wrote to config at `{}`.", config_path.display());
	}
}
