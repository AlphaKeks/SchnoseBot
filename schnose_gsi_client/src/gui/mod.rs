//! Main "App" struct used for egui.
//! The implementation for `App` is in `./gui_impl.rs`.

use {
	crate::{config::Config, logger::LogReceiver},
	chrono::Utc,
	color_eyre::Result,
	eframe::{
		egui::{
			style::Selection, Button, FontData, FontDefinitions, RichText, ScrollArea, Style,
			TextEdit, TextStyle, Ui, Visuals,
		},
		epaint::{FontFamily, FontId},
		HardwareAcceleration, NativeOptions, Theme,
	},
	rfd::FileDialog,
	std::{collections::BTreeMap, fs::File, sync::Arc},
	tokio::{sync::Mutex, task::JoinHandle},
	tracing::{error, info},
};

mod gui_impl;

pub mod colors;
pub mod state;

#[derive(Debug)]
pub struct Client {
	/// Global App config.
	pub config: Arc<Mutex<Config>>,

	/// For retrieving logging information. This is `None` if the application is logging to STDOUT.
	pub logger: Option<LogReceiver>,

	/// The currently selected tab of the application.
	pub current_tab: Tab,

	pub state: Arc<Mutex<Option<state::State>>>,

	/// A handle to shutdown the GSI Server running in the background.
	pub gsi_server_handle: Option<schnose_gsi::ServerHandle>,

	/// A handle to shutdown the Axum HTTP server running in the background.
	pub axum_server_handle: Option<JoinHandle<()>>,
}

#[derive(Debug, PartialEq)]
pub enum Tab {
	Main,
	Logs,
}

impl Client {
	pub const APP_NAME: &str = "schnose_gsi_client";
	pub const NORMAL_FONT: &str = "Quicksand";
	pub const MONOSPACE_FONT: &str = "Fira Code";

	#[tracing::instrument]
	pub async fn init(config: Config, logger: Option<LogReceiver>) -> Result<()> {
		let client = Self {
			config: Arc::new(Mutex::new(config)),
			logger,
			current_tab: Tab::Main,
			state: Arc::new(Mutex::new(None)),
			gsi_server_handle: None,
			axum_server_handle: None,
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
				Self::load_fonts(ctx);
				Self::load_visuals(ctx);
				Box::new(client)
			}),
		)
		.expect("Failed to run GUI.");

		Ok(())
	}

	#[tracing::instrument(skip(ctx))]
	fn load_fonts(ctx: &eframe::CreationContext) {
		let mut font_definitions = FontDefinitions::default();

		// Default font for most of the UI
		font_definitions.font_data.insert(
			String::from(Self::NORMAL_FONT),
			FontData::from_static(include_bytes!("../../assets/fonts/Quicksand.ttf")),
		);

		// Monospace font for codeblocks etc.
		font_definitions.font_data.insert(
			String::from(Self::MONOSPACE_FONT),
			FontData::from_static(include_bytes!("../../assets/fonts/Fira Code.ttf")),
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
		ctx.egui_ctx.set_style(Style {
			text_styles: BTreeMap::from_iter([
				(TextStyle::Heading, FontId::new(36.0, FontFamily::Proportional)),
				(TextStyle::Body, FontId::new(24.0, FontFamily::Proportional)),
				(TextStyle::Button, FontId::new(24.0, FontFamily::Proportional)),
				(TextStyle::Monospace, FontId::new(24.0, FontFamily::Monospace)),
			]),
			..Default::default()
		});
	}

	#[tracing::instrument(skip(ctx))]
	fn load_visuals(ctx: &eframe::CreationContext) {
		ctx.egui_ctx.set_visuals(Visuals {
			dark_mode: true,
			override_text_color: Some(colors::TEXT),
			selection: Selection {
				bg_fill: colors::SURFACE2,
				..Default::default()
			},
			hyperlink_color: colors::MAUVE,
			faint_bg_color: colors::MANTLE,
			extreme_bg_color: colors::CRUST,
			code_bg_color: colors::MANTLE,
			warn_fg_color: colors::PEACH,
			error_fg_color: colors::RED,
			window_fill: colors::BASE,
			panel_fill: colors::MANTLE,
			button_frame: true,
			slider_trailing_fill: true,
			..Default::default()
		});
	}

	pub const fn server_running(&self) -> bool {
		self.gsi_server_handle.is_some() && self.axum_server_handle.is_some()
	}

	#[tracing::instrument(skip(self))]
	pub fn spawn_gsi_server(&mut self) {
		self.gsi_server_handle =
			Some(crate::gsi::run_server(Arc::clone(&self.state), Arc::clone(&self.config)));
	}

	#[tracing::instrument(skip(self))]
	pub fn spawn_axum_server(&mut self) {
		self.axum_server_handle = Some(tokio::spawn(crate::server::run(Arc::clone(&self.state))));
	}

	#[tracing::instrument(skip(self))]
	pub fn kill_gsi_server(&mut self) {
		if let Some(handle) = self.gsi_server_handle.take() {
			handle.abort();
		}
	}

	#[tracing::instrument(skip(self))]
	pub fn kill_axum_server(&mut self) {
		if let Some(handle) = self.axum_server_handle.take() {
			handle.abort();
		}
	}

	/// The status text at the top of the screen indicating whether the GSI and Axum servers are
	/// running. Also creates a link to the URL for the Overlay, if the servers are running.
	pub fn render_status(&self, ui: &mut Ui) {
		ui.vertical_centered(|ui| {
			ui.label(
				match self.server_running() {
					true => RichText::new("Running").color(colors::GREEN),
					false => RichText::new("Stopped").color(colors::RED),
				}
				.heading(),
			);

			if self.server_running() {
				ui.hyperlink_to(
					"Open Overlay",
					format!("http://localhost:{}", crate::server::PORT),
				);
			}
		});
	}

	/// A button to open a file dialogue to select the user's `cfg` folder for CS:GO.
	pub fn render_cfg_prompt(config: &mut Config, ui: &mut Ui) {
		let button = ui.add(Button::new("Select your /csgo/cfg folder").fill(colors::SURFACE2));

		let mut current_folder = config
			.csgo_cfg_path
			.display()
			.to_string();

		if button.clicked() {
			if let Some(cfg_path) = FileDialog::new().pick_folder() {
				current_folder = cfg_path.display().to_string();
				config.csgo_cfg_path = cfg_path;
			}
		}

		button.on_hover_text(format!("Current folder: {current_folder}"));
	}

	/// Prompt to enter the user's API key (if they have one).
	/// This will be censored so streamers don't leak it.
	pub fn render_api_key_prompt(config: &mut Config, ui: &mut Ui) {
		ui.label("Enter your API Key: ");
		TextEdit::singleline(&mut config.schnose_api_key)
			.password(true)
			.show(ui);
	}

	/// Button to start/stop the GSI and Axum servers.
	pub fn render_run_button(&mut self, ui: &mut Ui) {
		if self.server_running() {
			let stop_button = Button::new("Stop GSI Server").fill(colors::SURFACE2);
			if ui.add(stop_button).clicked() {
				self.kill_axum_server();
				self.kill_gsi_server();
			}
		} else {
			let run_button = Button::new("Run GSI Server").fill(colors::SURFACE2);
			if ui.add(run_button).clicked() {
				self.spawn_gsi_server();
				self.spawn_axum_server();
			}
		}
	}

	/// The main window displaying [`render_api_key_prompt`] and [`render_run_button`].
	pub fn render_main(&mut self, ui: &mut Ui) {
		ui.vertical_centered(|ui| {
			tokio::task::block_in_place(|| {
				let mut config = self.config.blocking_lock();

				Self::render_cfg_prompt(&mut config, ui);
				Self::render_api_key_prompt(&mut config, ui);
			});

			ui.add_space(12.0);
			ui.separator();
			ui.add_space(12.0);

			self.render_run_button(ui);
			ui.add_space(12.0);
		});
	}

	/// The window showing the log output.
	pub fn render_logs(&mut self, ui: &mut Ui) {
		let Some(logger) = &mut self.logger else {
			ui.vertical_centered(|ui| ui.label("Logs are displayed on STDOUT."));
			return;
		};

		let mut logs = logger.current();

		// Delete old logs if the buffer grew too large.
		logs.truncate(u16::MAX as usize);

		let logs = LogReceiver::formatted(&logs);

		ScrollArea::new([true; 2])
			.auto_shrink([false; 2])
			.stick_to_bottom(true)
			.show_rows(ui, 12.0, logs.len(), |ui, range| {
				logs.into_iter()
					.skip(range.start)
					.take(range.len())
					.for_each(|log| {
						ui.horizontal(|ui| {
							ui.add_space(4.0);
							ui.label(log.timestamp);

							ui.add_space(4.0);
							ui.separator();
							ui.add_space(4.0);

							ui.label(log.level);

							ui.add_space(4.0);
							ui.separator();
							ui.add_space(4.0);

							ui.label(log.message);
							ui.add_space(4.0);
						});
					});
			});

		ui.add_space(4.0);
		ui.separator();
		self.render_save_logs_button(ui);
		ui.add_space(4.0);
	}

	/// Render a button to save current logs.
	pub fn render_save_logs_button(&mut self, ui: &mut Ui) {
		use std::io::Write;

		if ui.button("Save logs").clicked() {
			let date = Utc::now();
			let file_name = format!("{}-schnose-gsi.log", date.format("%Y%m%d%H%M%S"));

			let Some(log_path) = FileDialog::new().set_file_name(&file_name).save_file() else {
				return;
			};

			let mut file = match File::create(&log_path) {
				Ok(file) => file,
				Err(why) => return error!("Failed to open log file: {why:?}"),
			};

			let Some(logger) = &mut self.logger else {
				return error!("This UI should only ever be rendered if a logger is present.");
			};

			let logs = logger.current();

			match file.write_all(&logs) {
				Ok(()) => info!("Wrote logs to `{}`.", log_path.display()),
				Err(why) => {
					return error!("Failed to write logs to `{}`: {:?}", log_path.display(), why)
				}
			};
		}
	}
}
