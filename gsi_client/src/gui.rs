use {
	eframe::egui,
	gsi_client::gsi,
	std::{
		collections::BTreeMap,
		sync::mpsc::{Receiver, Sender},
	},
	tokio::runtime::Runtime,
	tracing::error,
};

#[derive(Debug)]
pub struct State {
	pub current_info: Option<gsi::Info>,
	pub auth_token: String,
	pub config: crate::Config,
	pub info_tx: Option<Sender<gsi::Info>>,
	pub info_rx: Receiver<gsi::Info>,
	pub gokz_client: gokz_rs::BlockingClient,
}

impl State {
	#[tracing::instrument]
	pub fn new(config: crate::Config, tx: Sender<gsi::Info>, rx: Receiver<gsi::Info>) -> Self {
		Self {
			current_info: None,
			auth_token: String::new(),
			config,
			info_tx: Some(tx),
			info_rx: rx,
			gokz_client: gokz_rs::BlockingClient::new(),
		}
	}

	#[tracing::instrument]
	pub fn info(&self) -> Option<gsi::Info> {
		self.current_info
			.as_ref()
			.map(ToOwned::to_owned)
	}

	#[tracing::instrument(skip(options))]
	pub fn run(self, options: eframe::NativeOptions) -> eframe::Result<()> {
		eframe::run_native("SchnoseBot CS:GO Watcher", options, Box::new(|_| Box::new(self)))
	}

	#[tracing::instrument(skip(ui))]
	pub fn render_status(&self, ui: &mut egui::Ui) {
		ui.heading("This is the status message!");
	}

	#[tracing::instrument(skip(ui))]
	pub fn render_path_box(&mut self, ui: &mut egui::Ui) {
		ui.label("Enter your csgo/cfg path here: ");
		ui.text_edit_singleline(&mut self.config.cfg_path);
	}

	#[tracing::instrument(skip(ui))]
	pub fn render_token_box(&mut self, ui: &mut egui::Ui) {
		ui.label("Enter your API token here: ");
		ui.text_edit_singleline(&mut self.auth_token);
	}

	#[tracing::instrument(skip(ui))]
	pub fn render_run_button(&mut self, ui: &mut egui::Ui) {
		if ui.button("Run watcher").clicked() {
			error!("STARTING GSI");

			let config = self.config.clone();
			let Some(tx) = self.info_tx.take() else {
				return;
			};

			// Spawn new thread for GSI and give it the sender to send information as it changes ingame.
			std::thread::spawn(move || {
				let mut gsi_runtime = Runtime::new().expect("Failed to spawn GSI runtime.");
				gsi_runtime.block_on(gsi::run(config.cfg_path, config.port, tx));
			});
		}
	}

	#[tracing::instrument(skip(ui))]
	pub fn render_current_info(&self, ui: &mut egui::Ui) {
		let info_json = match dbg!(&self.current_info) {
			Some(info) => {
				let (map_name, map_tier) = match &info.map {
					Some((name, tier)) => (name.to_owned(), format!("{} ({})", *tier as u8, tier)),
					None => (String::from("Unknown"), String::from("Unknown")),
				};
				format!(
					r#"
Info {{
    player: {}
    steam_id: {}
	mode: {}
	map_name: {}
	map_tier: {}
}}
"#,
					info.name,
					info.steam_id,
					info.mode
						.map_or_else(|| String::from("None"), |mode| mode.to_string()),
					map_name,
					map_tier
				)
			}
			None => String::new(),
		};

		// ui.code(info_json);
		ui.monospace(info_json);
	}
}

impl eframe::App for State {
	#[tracing::instrument(level = "WARN", skip(ctx, _frame))]
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		egui::CentralPanel::default().show(ctx, |ui| {
			let mut font_definitions = egui::FontDefinitions::default();

			font_definitions.font_data.insert(
				String::from("Quicksand"),
				egui::FontData::from_static(include_bytes!(
					"../static/fonts/Quicksand-Regular.ttf"
				)),
			);

			font_definitions.font_data.insert(
				String::from("Fira Code"),
				egui::FontData::from_static(include_bytes!(
					"../static/fonts/Fira Code Regular Nerd Font Complete Mono.ttf"
				)),
			);

			font_definitions
				.families
				.entry(egui::FontFamily::Proportional)
				.or_default()
				.insert(0, String::from("Quicksand"));

			font_definitions
				.families
				.entry(egui::FontFamily::Monospace)
				.or_default()
				.insert(0, String::from("Fira Code"));

			ctx.set_fonts(font_definitions);

			let style = egui::Style {
				text_styles: BTreeMap::from_iter([
					(
						egui::TextStyle::Heading,
						egui::FontId::new(36.0, egui::FontFamily::Proportional),
					),
					(
						egui::TextStyle::Name("Heading2".into()),
						egui::FontId::new(32.0, egui::FontFamily::Proportional),
					),
					(
						egui::TextStyle::Name("ContextHeading".into()),
						egui::FontId::new(28.0, egui::FontFamily::Proportional),
					),
					(
						egui::TextStyle::Body,
						egui::FontId::new(24.0, egui::FontFamily::Proportional),
					),
					(
						egui::TextStyle::Monospace,
						egui::FontId::new(20.0, egui::FontFamily::Monospace),
					),
					(
						egui::TextStyle::Button,
						egui::FontId::new(24.0, egui::FontFamily::Proportional),
					),
					(
						egui::TextStyle::Small,
						egui::FontId::new(16.0, egui::FontFamily::Proportional),
					),
				]),
				..Default::default()
			};

			ctx.set_style(style);

			let visuals = egui::Visuals {
				panel_fill: egui::Color32::from_rgb(30, 30, 46),
				code_bg_color: egui::Color32::from_rgb(24, 24, 37),
				override_text_color: Some(egui::Color32::from_rgb(205, 214, 244)),
				extreme_bg_color: egui::Color32::from_rgb(17, 17, 27),
				..Default::default()
			};

			ctx.set_visuals(visuals);

			if let Ok(info) = self.info_rx.try_recv() {
				self.current_info = Some(info);
			}

			ui.vertical_centered(|ui| {
				self.render_status(ui);
				self.render_path_box(ui);
				self.render_token_box(ui);
				self.render_run_button(ui);
			});

			self.render_current_info(ui);
		});
	}
}
