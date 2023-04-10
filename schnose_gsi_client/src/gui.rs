use {
	crate::logger::Event,
	chrono::Utc,
	eframe::{
		egui::{
			self, style::Selection, Button, CentralPanel, FontData, FontDefinitions, RichText,
			ScrollArea, Style, TextEdit, TextStyle, TopBottomPanel, Ui, Visuals,
		},
		epaint::{Color32, FontFamily, FontId},
		HardwareAcceleration, NativeOptions, Theme,
	},
	rfd::FileDialog,
	schnose_gsi_client::{
		config::Config,
		gsi::{self, CSGOReport},
		server,
	},
	serde::Serialize,
	std::{collections::BTreeMap, fs::File, io::Write},
	tokio::{
		sync::mpsc::{self, UnboundedReceiver},
		task::JoinHandle,
	},
	tracing::{error, info},
};

#[derive(Debug, Serialize, PartialEq)]
pub enum Tab {
	Main,
	Logs,
}

#[derive(Debug, Serialize)]
pub struct GsiGui {
	pub csgo_report: Option<CSGOReport>,
	pub csgo_cfg_folder: String,
	pub config: Config,
	pub gsi_server_running: bool,
	pub current_tab: Tab,

	#[serde(skip)]
	pub log_receiver: UnboundedReceiver<Vec<u8>>,
	#[serde(skip)]
	pub logs: Vec<Event>,

	#[serde(skip)]
	pub gsi_handle: Option<schnose_gsi::ServerHandle>,
	#[serde(skip)]
	pub axum_handle: Option<JoinHandle<()>>,
}

impl GsiGui {
	pub const APP_NAME: &str = "schnose_gsi_client";
	pub const NORMAL_FONT: &str = "Quicksand";
	pub const MONOSPACE_FONT: &str = "Fira Code";

	pub const _ROSEWATER: Color32 = Color32::from_rgb(245, 224, 220);
	pub const _FLAMINGO: Color32 = Color32::from_rgb(242, 205, 205);
	pub const _PINK: Color32 = Color32::from_rgb(245, 194, 231);
	pub const MAUVE: Color32 = Color32::from_rgb(203, 166, 247);
	pub const RED: Color32 = Color32::from_rgb(243, 139, 168);
	pub const _MAROON: Color32 = Color32::from_rgb(235, 160, 172);
	pub const PEACH: Color32 = Color32::from_rgb(250, 179, 135);
	pub const YELLOW: Color32 = Color32::from_rgb(249, 226, 175);
	pub const GREEN: Color32 = Color32::from_rgb(166, 227, 161);
	pub const TEAL: Color32 = Color32::from_rgb(148, 226, 213);
	pub const _SKY: Color32 = Color32::from_rgb(137, 220, 235);
	pub const _SAPPHIRE: Color32 = Color32::from_rgb(116, 199, 236);
	pub const BLUE: Color32 = Color32::from_rgb(137, 180, 250);
	pub const LAVENDER: Color32 = Color32::from_rgb(180, 190, 254);
	pub const TEXT: Color32 = Color32::from_rgb(205, 214, 244);
	pub const _SUBTEXT1: Color32 = Color32::from_rgb(186, 194, 222);
	pub const _SUBTEXT0: Color32 = Color32::from_rgb(166, 173, 200);
	pub const _OVERLAY2: Color32 = Color32::from_rgb(147, 153, 178);
	pub const _OVERLAY1: Color32 = Color32::from_rgb(127, 132, 156);
	pub const _OVERLAY0: Color32 = Color32::from_rgb(108, 112, 134);
	pub const SURFACE2: Color32 = Color32::from_rgb(88, 91, 112);
	pub const _SURFACE1: Color32 = Color32::from_rgb(69, 71, 90);
	pub const _SURFACE0: Color32 = Color32::from_rgb(49, 50, 68);
	pub const BASE: Color32 = Color32::from_rgb(30, 30, 46);
	pub const MANTLE: Color32 = Color32::from_rgb(24, 24, 37);
	pub const CRUST: Color32 = Color32::from_rgb(17, 17, 27);
	pub const POGGERS: Color32 = Color32::from_rgb(116, 128, 194);

	#[tracing::instrument]
	pub async fn init(
		config: Config,
		log_receiver: UnboundedReceiver<Vec<u8>>,
	) -> eframe::Result<()> {
		let csgo_cfg_folder = config
			.csgo_cfg_path
			.display()
			.to_string();

		let gui = Self {
			csgo_report: None,
			csgo_cfg_folder,
			config,
			current_tab: Tab::Main,
			log_receiver,
			logs: Vec::new(),
			gsi_server_running: false,
			gsi_handle: None,
			axum_handle: None,
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
			FontData::from_static(include_bytes!("../assets/fonts/Quicksand.ttf")),
		);

		// Monospace font for codeblocks etc.
		font_definitions.font_data.insert(
			String::from(Self::MONOSPACE_FONT),
			FontData::from_static(include_bytes!("../assets/fonts/Fira Code.ttf")),
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
			override_text_color: Some(Self::TEXT),
			selection: Selection {
				bg_fill: Self::SURFACE2,
				..Default::default()
			},
			hyperlink_color: Self::MAUVE,
			faint_bg_color: Self::MANTLE,
			extreme_bg_color: Self::CRUST,
			code_bg_color: Self::MANTLE,
			warn_fg_color: Self::PEACH,
			error_fg_color: Self::RED,
			window_fill: Self::BASE,
			panel_fill: Self::MANTLE,
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
					.color(Self::POGGERS),
			);

			ui.heading(match self.gsi_server_running {
				true => RichText::new("Running")
					.color(Self::GREEN)
					.heading(),
				false => RichText::new("Stopped")
					.color(Self::RED)
					.heading(),
			});

			if self.gsi_server_running {
				ui.hyperlink_to("Open Overlay", "http://localhost:9999");
			}

			ui.add_space(12.0);
		});
	}

	pub fn render_cfg_path_prompt(&mut self, ui: &mut Ui) {
		ui.vertical_centered(|ui| {
			let button = ui.add(Button::new("Select your /csgo/cfg folder").fill(Self::SURFACE2));

			if button.clicked() {
				if let Some(path) = FileDialog::new().pick_folder() {
					let path = path.display().to_string();
					self.csgo_cfg_folder = path.clone();
					self.config.csgo_cfg_path = path.into();
				}
			}

			button.on_hover_text(format!("Current folder: {}", self.csgo_cfg_folder));
		});

		ui.add_space(12.0);
	}

	pub fn render_api_key_prompt(&mut self, ui: &mut Ui) {
		ui.vertical_centered(|ui| {
			ui.label("Enter your API Key: ");
			TextEdit::singleline(&mut self.config.schnose_api_key)
				.password(true)
				.show(ui);
		});

		ui.add_space(12.0);
	}

	#[tracing::instrument]
	fn run_server(&mut self) {
		let (axum_sender, axum_receiver) = mpsc::unbounded_channel();

		self.gsi_handle = Some(
			gsi::run_server(axum_sender, self.config.clone()).expect("Failed to run GSI server"),
		);

		self.axum_handle = Some(tokio::spawn(server::run(axum_receiver)));

		self.gsi_server_running = true;

		info!("Started GSI server");
	}

	#[tracing::instrument]
	fn stop_server(&mut self) {
		let gsi_handle = self
			.gsi_handle
			.take()
			.expect("This should only ever be called after the server has been started.");

		let axum_handle = self
			.axum_handle
			.take()
			.expect("This should only ever be called after the server has been started.");

		gsi_handle.abort();
		axum_handle.abort();

		self.gsi_server_running = false;

		info!("Stopped GSI server");
	}

	pub fn render_run_button(&mut self, ui: &mut Ui) {
		ui.vertical_centered(|ui| {
			match self.gsi_server_running {
				false => {
					let run_button = Button::new("Run GSI server").fill(Self::SURFACE2);
					if ui.add(run_button).clicked() {
						self.run_server();
					}
				}
				true => {
					let stop_button = Button::new("Stop GSI server").fill(Self::SURFACE2);
					if ui.add(stop_button).clicked() {
						self.stop_server();
					}
				}
			};
		});
	}

	pub fn render_main(&mut self, ui: &mut Ui) {
		self.render_cfg_path_prompt(ui);
		self.render_api_key_prompt(ui);
		self.render_run_button(ui);
	}

	pub fn render_logs(&mut self, ui: &mut Ui) {
		if ui.button("Save logs").clicked() {
			let date = Utc::now();
			let file_name = format!("{}-schnose-gsi.log", date.format("%Y%m%d%H%M%S"));

			if let Some(log_path) = rfd::FileDialog::new()
				.set_file_name(&file_name)
				.save_file()
			{
				match File::create(&log_path) {
					Ok(mut file) => {
						match serde_json::to_vec(&self.logs) {
							Ok(json) => {
								match file.write_all(&json) {
									Ok(()) => {
										info!("Wrote config file to `{}`.", log_path.display())
									}
									Err(why) => error!(
										"Failed to save logs to `{}`. {:?}",
										log_path.display(),
										why
									),
								};
							}
							Err(why) => error!("Failed to convert logs to json: {why:?}"),
						};
					}
					Err(why) => error!("Failed to save logs: {why:?}"),
				};
			}
		}

		ui.add_space(4.0);
		ui.separator();
		ui.add_space(4.0);

		ScrollArea::new([true; 2])
			.auto_shrink([false; 2])
			.stick_to_bottom(true)
			.always_show_scroll(true)
			.show_rows(ui, 12.0, self.logs.len(), |ui, range| {
				self.logs
					.iter()
					.skip(range.start)
					.take(range.len())
					.for_each(|event| event.render(ui));
			});
	}
}

impl eframe::App for GsiGui {
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		let new_logs = self
			.log_receiver
			.try_recv()
			.unwrap_or_default();

		self.logs.extend(
			String::from_utf8_lossy(&new_logs)
				.lines()
				.filter_map(|line| serde_json::from_str::<Event>(line).ok()),
		);

		if std::mem::size_of_val(&self.logs) > 1_000_000 {
			let mid = self.logs.len() / 2;
			self.logs.drain(0..mid);
		}

		TopBottomPanel::top("heading-panel").show(ctx, |ui| {
			ui.horizontal(|ui| {
				ui.selectable_value(&mut self.current_tab, Tab::Main, "Main");
				ui.selectable_value(&mut self.current_tab, Tab::Logs, "Logs");
			});
			self.render_heading(ui);
		});

		CentralPanel::default().show(ctx, |ui| {
			match self.current_tab {
				Tab::Main => self.render_main(ui),
				Tab::Logs => self.render_logs(ui),
			};
		});
	}

	fn save(&mut self, _storage: &mut dyn eframe::Storage) {
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
