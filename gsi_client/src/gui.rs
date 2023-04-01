use {
	crate::http_server,
	eframe::egui,
	gsi_client::gsi,
	std::{
		collections::BTreeMap,
		sync::{Arc, Mutex},
	},
	tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender},
	tracing::info,
};

/// The GUI state object.
#[derive(Debug)]
pub struct State {
	pub current_info: Arc<Mutex<Option<gsi::Info>>>,
	pub config: crate::Config,
	/// This will be sent to the GSI thread when clicking a button.
	pub info_tx: Option<UnboundedSender<gsi::Info>>,
	pub info_rx: UnboundedReceiver<gsi::Info>,
	pub gsi_running: bool,
	pub gokz_client: gokz_rs::BlockingClient,
}

impl State {
	#[tracing::instrument]
	pub fn new(
		config: crate::Config,
		tx: UnboundedSender<gsi::Info>,
		rx: UnboundedReceiver<gsi::Info>,
	) -> Self {
		Self {
			current_info: Arc::new(Mutex::new(None)),
			config,
			info_tx: Some(tx),
			info_rx: rx,
			gsi_running: false,
			gokz_client: gokz_rs::BlockingClient::new(),
		}
	}

	#[tracing::instrument]
	pub fn info(&self) -> Option<gsi::Info> {
		self.current_info
			.try_lock()
			.ok()
			.and_then(|lock| (*lock).clone())
	}

	/// Start the GUI.
	#[tracing::instrument(skip(options))]
	pub fn run(self, options: eframe::NativeOptions) -> eframe::Result<()> {
		let current_info = Arc::clone(&self.current_info);
		tokio::spawn(http_server::run(current_info));
		eframe::run_native("SchnoseBot CS:GO Watcher", options, Box::new(|_| Box::new(self)))
	}

	/// Status message at the top of the screen.
	#[tracing::instrument(skip(ui))]
	pub fn render_status(&self, ui: &mut egui::Ui) {
		ui.heading(match self.gsi_running {
			true => egui::RichText::new("Running").color(egui::Color32::from_rgb(166, 227, 161)),
			false => egui::RichText::new("Stopped").color(egui::Color32::from_rgb(243, 139, 168)),
		});
	}

	/// The text box for putting in the CS:GO path.
	#[tracing::instrument(skip(ui))]
	pub fn render_path_box(&mut self, ui: &mut egui::Ui) {
		ui.label("Current CFG folder:");
		ui.code(format!("{}\n", self.config.cfg_path));

		if ui.button("Choose new folder").clicked() {
			if let Some(path) = rfd::FileDialog::new().pick_folder() {
				self.config.cfg_path = path
					.as_os_str()
					.to_string_lossy()
					.to_string();
			}
		}
	}

	/// The text box for putting in the API key.
	#[tracing::instrument(skip(ui))]
	pub fn render_api_key_box(&mut self, ui: &mut egui::Ui) {
		ui.label("\nEnter your API key here: ");
		ui.text_edit_singleline(&mut self.config.api_key);
	}

	/// The button to launch the GSI thread.
	#[tracing::instrument(skip(ui))]
	pub fn render_run_button(&mut self, ui: &mut egui::Ui) {
		if !self.gsi_running {
			let button = ui.button("Run watcher");
			if button.clicked() {
				info!("STARTING GSI");

				// Don't spawn more than 1 client.
				let Some(tx) = self.info_tx.take() else {
					return;
				};

				// Spawn new thread for GSI and give it the sender to send information as it changes
				// ingame.
				tokio::spawn(gsi::run(
					self.config.cfg_path.clone(),
					self.config.api_key.clone(),
					self.config.port,
					tx,
				));

				self.gsi_running = true;
			}
		}
	}

	/// Pretty print the current [`Info`] in the GUI.
	#[tracing::instrument(skip(ui))]
	pub fn render_current_info(&self, ui: &mut egui::Ui) {
		let Ok(current_info) = self.current_info.try_lock() else {
			return;
		};

		let info_json = match &*current_info {
			Some(info) => {
				let (map_name, map_tier) = match &info.map {
					Some(map) => (map.name.clone(), format!("{} ({})", map.tier as u8, map.tier)),
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
					info.player_name,
					info.steam_id,
					info.mode
						.map_or_else(|| String::from("None"), |mode| mode.to_string()),
					map_name,
					map_tier
				)
			}
			None => String::new(),
		};

		ui.monospace(info_json);
	}

	/// Load fonts and apply them to the UI:
	///   - Quicksand (for normal text)
	///   - Fira Code (for the pretty-printed [`Info`])
	#[tracing::instrument(skip(ctx))]
	pub fn setup_fonts(&self, ctx: &egui::Context) {
		let mut font_definitions = egui::FontDefinitions::default();

		font_definitions.font_data.insert(
			String::from("Quicksand"),
			egui::FontData::from_static(include_bytes!("../static/fonts/Quicksand-Regular.ttf")),
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

		// Adjust font sizes.

		let style = egui::Style {
			text_styles: BTreeMap::from_iter([
				(egui::TextStyle::Heading, egui::FontId::new(36.0, egui::FontFamily::Proportional)),
				(
					egui::TextStyle::Name("Heading2".into()),
					egui::FontId::new(32.0, egui::FontFamily::Proportional),
				),
				(
					egui::TextStyle::Name("ContextHeading".into()),
					egui::FontId::new(28.0, egui::FontFamily::Proportional),
				),
				(egui::TextStyle::Body, egui::FontId::new(24.0, egui::FontFamily::Proportional)),
				(egui::TextStyle::Monospace, egui::FontId::new(20.0, egui::FontFamily::Monospace)),
				(egui::TextStyle::Button, egui::FontId::new(24.0, egui::FontFamily::Proportional)),
				(egui::TextStyle::Small, egui::FontId::new(16.0, egui::FontFamily::Proportional)),
			]),
			..Default::default()
		};

		ctx.set_style(style);
	}

	/// Setup colors for the UI.
	#[tracing::instrument(skip(ctx))]
	pub fn setup_colors(&self, ctx: &egui::Context) {
		let visuals = egui::Visuals {
			panel_fill: egui::Color32::from_rgb(30, 30, 46),
			code_bg_color: egui::Color32::from_rgb(24, 24, 37),
			override_text_color: Some(egui::Color32::from_rgb(205, 214, 244)),
			extreme_bg_color: egui::Color32::from_rgb(17, 17, 27),
			..Default::default()
		};

		ctx.set_visuals(visuals);
	}
}

impl eframe::App for State {
	/// This function will run every single frame to draw the UI.
	#[tracing::instrument(level = "WARN", skip(ctx, _frame))]
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		egui::CentralPanel::default().show(ctx, |ui| {
			// Ask for new [`Info`] struct from the GSI thread.
			if let Ok(info) = self.info_rx.try_recv() {
				// If they are different, update the state and send the new info to SchnoseAPI.
				let Ok(mut current_info) = self.current_info.try_lock() else {
					return;
				};

				if current_info.as_ref() != Some(&info) {
					*current_info = Some(info);
				}
			}

			self.setup_fonts(ctx);
			self.setup_colors(ctx);

			// Render the UI.
			ui.vertical_centered(|ui| {
				self.render_status(ui);
				self.render_path_box(ui);
				self.render_api_key_box(ui);
				self.render_run_button(ui);
			});

			self.render_current_info(ui);
		});
	}
}
