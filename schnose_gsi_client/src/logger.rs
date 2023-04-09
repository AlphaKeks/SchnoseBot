use {
	crate::gui::GsiGui,
	eframe::egui::{RichText, Ui},
	serde::{Deserialize, Serialize},
	std::{io, ops::Deref},
	tokio::sync::mpsc::UnboundedSender,
	tracing::error,
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

impl std::fmt::Display for Event {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let (date, time) = self
			.timestamp
			.split_once('T')
			.ok_or(std::fmt::Error::default())?;

		let (time, _) = time
			.split_once('.')
			.ok_or(std::fmt::Error::default())?;

		let timestamp = format!("[{date} | {time}]");

		f.write_fmt(format_args!(
			"({}) {} @ {}:{} | {:?}",
			self.level, timestamp, self.filename, self.line_number, self.body
		))
	}
}

impl Event {
	pub fn render(&self, ui: &mut Ui) {
		let event = self.to_string();

		let (level, event) = event
			.split_once(' ')
			.unwrap_or_default();

		let level = match level {
			"(TRACE)" => RichText::new("(TRACE) ").color(GsiGui::TEAL),
			"(DEBUG)" => RichText::new("(DEBUG) ").color(GsiGui::BLUE),
			"(INFO)" => RichText::new("(INFO) ").color(GsiGui::GREEN),
			"(WARN)" => RichText::new("(WARN) ").color(GsiGui::YELLOW),
			"(ERROR)" => RichText::new("(ERROR) ").color(GsiGui::RED),
			level => RichText::new(format!("({level}) ")).color(GsiGui::MAUVE),
		};

		let (meta, body) = event
			.rsplit_once('|')
			.unwrap_or_default();

		let text = RichText::new(format!("{meta}\n\t{body}")).color(GsiGui::LAVENDER);

		ui.horizontal_wrapped(|ui| {
			ui.label(level);
			ui.label(text);
		});
	}
}
