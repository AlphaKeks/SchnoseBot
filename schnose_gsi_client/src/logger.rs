use {
	crate::gui::GsiGui,
	eframe::egui::{RichText, Ui},
	serde::{Deserialize, Serialize},
	std::{io, ops::Deref},
	tokio::sync::mpsc::UnboundedSender,
	tracing::{debug, error},
};

#[derive(Debug)]
pub struct Logger {
	sender: UnboundedSender<Vec<u8>>,
}

impl Logger {
	pub fn new(sender: UnboundedSender<Vec<u8>>) -> Self {
		Self { sender }
	}
}

impl Deref for Logger {
	type Target = UnboundedSender<Vec<u8>>;

	fn deref(&self) -> &Self::Target {
		&self.sender
	}
}

impl io::Write for &Logger {
	fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
		if let Err(why) = self.send(Vec::from(buf)) {
			error!("Failed to send log information: {why:?}");
			return Err(io::Error::new(
				io::ErrorKind::Other,
				"Failed to send log data through channel.",
			));
		}

		Ok(buf.len())
	}

	fn flush(&mut self) -> io::Result<()> {
		Ok(())
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
	pub timestamp: String,
	pub level: String,
	pub filename: String,
	pub line_number: usize,
	#[serde(flatten)]
	pub body: serde_json::Value,
}

impl Event {
	pub fn render(&self, ui: &mut Ui) {
		let Ok(message) = serde_json::to_string_pretty(&self.body["fields"]["message"]) else {
			debug!("no message");
			return;
		};

		let message = RichText::new(message)
			.color(GsiGui::LAVENDER)
			.monospace();

		let (date, time) = self
			.timestamp
			.split_once('T')
			.unwrap_or_default();
		let (time, _) = time.split_once('.').unwrap_or_default();
		let timestamp = format!("{} {}", date.replace('-', "/"), time);

		let timestamp = RichText::new(timestamp)
			.color(GsiGui::POGGERS)
			.monospace();

		let level = match self.level.as_str() {
			"TRACE" => RichText::new("[TRACE]").color(GsiGui::TEAL),
			"DEBUG" => RichText::new("[DEBUG]").color(GsiGui::BLUE),
			"INFO" => RichText::new("[INFO] ").color(GsiGui::GREEN),
			"WARN" => RichText::new("[WARN] ").color(GsiGui::YELLOW),
			"ERROR" => RichText::new("[ERROR]").color(GsiGui::RED),
			level => RichText::new(format!("[{level}]")).color(GsiGui::MAUVE),
		}
		.monospace();

		ui.horizontal_top(|ui| {
			ui.add_space(4.0);
			ui.label(timestamp);
			ui.add_space(4.0);
			ui.separator();

			ui.horizontal(|ui| {
				ui.add_space(4.0);
				ui.label(level);
				ui.add_space(4.0);
				ui.separator();
			});

			ui.add_space(4.0);
			ui.label(message);
			ui.add_space(4.0);
		});
	}
}
