use {
	eframe::{
		egui::{
			self, style::Selection, CentralPanel, FontData, FontDefinitions, RichText, Style,
			TextStyle, TopBottomPanel, Ui, Visuals,
		},
		epaint::{Color32, FontFamily, FontId},
		HardwareAcceleration, NativeOptions, Theme,
	},
	rfd::FileDialog,
	schnose_gsi_client::{
		config::Config,
		gsi::{self, CSGOReport},
	},
	serde::Serialize,
	std::{collections::BTreeMap, fs::File},
	tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender},
	tracing::{error, info},
};

#[derive(Debug, Serialize)]
pub struct GsiGui {
	pub csgo_report: Option<CSGOReport>,
	#[serde(skip)]
	pub gsi_sender: UnboundedSender<CSGOReport>,
	#[serde(skip)]
	pub gsi_receiver: UnboundedReceiver<CSGOReport>,
	pub csgo_cfg_folder: String,
	pub config: Config,
	pub gsi_server_running: bool,
}

impl GsiGui {
	const APP_NAME: &str = "schnose_gsi_client";
	const NORMAL_FONT: &str = "Quicksand";
	const MONOSPACE_FONT: &str = "Fira Code";

	#[tracing::instrument]
	pub async fn init(
		gsi_sender: UnboundedSender<CSGOReport>,
		gsi_receiver: UnboundedReceiver<CSGOReport>,
		config: Config,
	) -> eframe::Result<()> {
		let csgo_cfg_folder = config
			.csgo_cfg_path
			.display()
			.to_string();

		let gui = Self {
			csgo_report: None,
			gsi_sender,
			gsi_receiver,
			csgo_cfg_folder,
			config,
			gsi_server_running: false,
		};

		let native_options = NativeOptions {
			always_on_top: false,
			decorated: true,
			fullscreen: false,
			drag_and_drop_support: true,
			resizable: true,
			transparent: true,
			mouse_passthrough: false,
			vsync: true,
			hardware_acceleration: HardwareAcceleration::Preferred,
			follow_system_theme: false,
			default_theme: Theme::Dark,
			centered: true,
			..Default::default()
		};

		eframe::run_native(
			Self::APP_NAME,
			native_options,
			Box::new(|ctx| {
				gui.load_fonts(ctx);
				gui.load_colors(ctx);

				Box::new(gui)
			}),
		)
	}

	#[tracing::instrument(skip(ctx))]
	pub fn load_fonts(&self, ctx: &eframe::CreationContext) {
		let mut font_definitions = FontDefinitions::default();

		// Default font for most of the UI
		font_definitions.font_data.insert(
			String::from(Self::NORMAL_FONT),
			FontData::from_static(include_bytes!("../static/fonts/Quicksand.ttf")),
		);

		// Monospace font for codeblocks etc.
		font_definitions.font_data.insert(
			String::from(Self::MONOSPACE_FONT),
			FontData::from_static(include_bytes!("../static/fonts/Fira Code.ttf")),
		);

		font_definitions
			.families
			.entry(FontFamily::Proportional)
			.or_default()
			.insert(0, String::from(Self::NORMAL_FONT));

		font_definitions
			.families
			.entry(FontFamily::Monospace)
			.or_default()
			.insert(0, String::from(Self::MONOSPACE_FONT));

		ctx.egui_ctx.set_fonts(font_definitions);

		let style = Style {
			text_styles: BTreeMap::from_iter([
				(TextStyle::Heading, FontId::new(36.0, FontFamily::Proportional)),
				(TextStyle::Body, FontId::new(24.0, FontFamily::Proportional)),
				(TextStyle::Button, FontId::new(24.0, FontFamily::Proportional)),
				(TextStyle::Monospace, FontId::new(24.0, FontFamily::Monospace)),
			]),
			..Default::default()
		};

		ctx.egui_ctx.set_style(style);
	}

	#[tracing::instrument(skip(ctx))]
	pub fn load_colors(&self, ctx: &eframe::CreationContext) {
		let visuals = Visuals {
			dark_mode: true,
			override_text_color: Some(Color32::from_rgb(205, 214, 244)),
			selection: Selection {
				bg_fill: Color32::from_rgb(69, 71, 90),
				..Default::default()
			},
			hyperlink_color: Color32::from_rgb(203, 166, 247),
			faint_bg_color: Color32::from_rgb(49, 50, 68),
			extreme_bg_color: Color32::from_rgb(17, 17, 27),
			code_bg_color: Color32::from_rgb(24, 24, 37),
			warn_fg_color: Color32::from_rgb(250, 179, 135),
			error_fg_color: Color32::from_rgb(243, 139, 168),
			window_fill: Color32::from_rgb(30, 30, 46),
			panel_fill: Color32::from_rgb(24, 24, 37),
			button_frame: true,
			slider_trailing_fill: true,
			..Default::default()
		};

		ctx.egui_ctx.set_visuals(visuals);
	}

	pub fn render_heading(&self, ui: &mut Ui) {
		ui.vertical_centered(|ui| {
			ui.heading(
				RichText::new("SchnoseBot GSI Client")
					.heading()
					.color(Color32::from_rgb(116, 128, 194)),
			);

			ui.heading(match self.gsi_server_running {
				true => RichText::new(" (Running)")
					.color(Color32::from_rgb(166, 227, 161))
					.heading(),
				false => RichText::new(" (Stopped)")
					.color(Color32::from_rgb(243, 139, 168))
					.heading(),
			});

			ui.add_space(12.0);
		});
	}

	pub fn render_cfg_path_prompt(&mut self, ui: &mut Ui) {
		ui.vertical_centered(|ui| {
			if ui
				.button("Select your /csgo/cfg folder")
				.clicked()
			{
				if let Some(path) = FileDialog::new().pick_folder() {
					let path = path.display().to_string();
					self.csgo_cfg_folder = path.clone();
					self.config.csgo_cfg_path = path.into();
				}
			}

			ui.code(format!("Current folder: {}\n", self.csgo_cfg_folder));
		});
	}

	pub fn render_api_key_prompt(&mut self, ui: &mut Ui) {
		ui.vertical_centered(|ui| {
			ui.label("Enter your API Key: ");
			ui.text_edit_singleline(&mut self.config.schnose_api_key);
		});
	}

	#[tracing::instrument(skip(ui))]
	pub fn render_run_button(&mut self, ui: &mut Ui) {
		if self.gsi_server_running {
			return;
		}

		ui.vertical_centered(|ui| {
			if ui.button("Run GSI server.").clicked() {
				// Spawn a thread to listen for CS:GO events and send them back to
				// the GUI through a channel.
				tokio::spawn(gsi::run_server(self.gsi_sender.clone(), self.config.clone()));
				self.gsi_server_running = true;
			}
		});
	}

	pub fn render_csgo_report(&mut self, ui: &mut Ui) {
		// If there is a new report in the channel, render that. Otherwise render whatever is
		// currently stored in state.
		let csgo_report = match self.gsi_receiver.try_recv() {
			Ok(report) => {
				self.csgo_report = Some(report.clone());
				Some(report)
			}
			Err(_) => self.csgo_report.clone(),
		};

		match csgo_report {
			Some(report) => {
				let report_text = serde_json::to_string_pretty(&report)
					.unwrap_or_else(|_| String::from("can't parse report"));
				ui.code(report_text);
			}
			None => {
				ui.vertical_centered(|ui| {
					ui.heading("No info");
				});
			}
		};
	}
}

impl eframe::App for GsiGui {
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		TopBottomPanel::top("heading-panel").show(ctx, |ui| {
			self.render_heading(ui);
		});

		CentralPanel::default().show(ctx, |ui| {
			self.render_cfg_path_prompt(ui);
			self.render_run_button(ui);
			self.render_csgo_report(ui);
			self.render_api_key_prompt(ui);
		});
	}

	fn save(&mut self, _storage: &mut dyn eframe::Storage) {
		use std::io::Write;

		let Ok(config_path) = Config::get_path() else {
			return error!("Failed to locate config file.");
		};

		let Ok(mut config_file) = File::options().write(true).create(true).open(&config_path) else {
			return error!("Failed to open config file at `{}`.", self.csgo_cfg_folder);
		};

		let Ok(config_as_toml) = toml::to_string_pretty(&self.config) else {
			return error!("Failed to serialize config as toml. ({:#?})", self.config);
		};

		if let Err(why) = config_file.write_all(config_as_toml.as_bytes()) {
			return error!("Failed to write to config file: {why:#?}");
		}

		info!("Successfully wrote to config at `{}`.", config_path.display());
	}
}
