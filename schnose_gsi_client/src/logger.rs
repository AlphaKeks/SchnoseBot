use {
	crate::gui::colors,
	eframe::egui::RichText,
	serde_json::Value as JsonValue,
	std::ops::Deref,
	tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
	tracing::error,
};

#[derive(Debug)]
pub struct LogSender {
	sender: UnboundedSender<Vec<u8>>,
}

#[derive(Debug)]
pub struct LogReceiver {
	receiver: UnboundedReceiver<Vec<u8>>,
	buffer: Vec<u8>,
}

#[tracing::instrument]
pub fn new() -> (LogSender, LogReceiver) {
	let (sender, receiver) = mpsc::unbounded_channel();
	(LogSender { sender }, LogReceiver { receiver, buffer: Vec::new() })
}

pub struct Log {
	pub message: RichText,
	pub timestamp: RichText,
	pub level: RichText,
}

impl LogReceiver {
	/// Get a copy of the current buffer's contents.
	pub fn current(&mut self) -> Vec<u8> {
		if let Ok(new_logs) = self.receiver.try_recv() {
			self.buffer.extend(new_logs);

			// Just to make sure we don't grow to infinity and beyond.
			self.buffer.truncate(1024 * 1_000_000);
		}

		self.buffer.clone()
	}

	/// Takes in a bunch of logs, returning a bunch of nicely formatted logs.
	pub fn formatted(logs: &[u8]) -> Vec<Log> {
		String::from_utf8_lossy(&logs)
			.lines()
			.into_iter()
			.flat_map(|log_line| serde_json::from_str::<JsonValue>(log_line))
			.filter_map(|json| {
				let message = json
					.get("fields")?
					.get("message")?
					.as_str()?;
				let message = RichText::new(message)
					.color(colors::LAVENDER)
					.monospace();

				let (date, time) = json
					.get("timestamp")?
					.as_str()?
					.split_once('T')?;
				let (time, _) = time.split_once('.')?;
				let timestamp = RichText::new(format!("{} {}", date.replace('-', "/"), time))
					.color(colors::POGGERS)
					.monospace();

				let level = match json.get("level")?.as_str()? {
					"TRACE" => RichText::new("[TRACE]").color(colors::TEAL),
					"DEBUG" => RichText::new("[DEBUG]").color(colors::BLUE),
					"INFO" => RichText::new("[INFO] ").color(colors::GREEN),
					"WARN" => RichText::new("[WARN] ").color(colors::YELLOW),
					"ERROR" => RichText::new("[ERROR]").color(colors::RED),
					level => RichText::new(format!("[{level}]")).color(colors::MAUVE),
				}
				.monospace();

				Some(Log { message, timestamp, level })
			})
			.collect()
	}
}

impl Deref for LogReceiver {
	type Target = Vec<u8>;

	fn deref(&self) -> &Self::Target {
		&self.buffer
	}
}

impl std::io::Write for &LogSender {
	fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
		match self.sender.send(Vec::from(buf)) {
			Ok(()) => Ok(buf.len()),
			Err(why) => {
				let message = format!("Failed to send new logs: {why:?}");
				error!(message);
				Err(std::io::Error::new(std::io::ErrorKind::Other, message))
			}
		}
	}

	fn flush(&mut self) -> std::io::Result<()> {
		Ok(())
	}
}
