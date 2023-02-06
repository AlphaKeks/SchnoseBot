use gokz_rs::{records::Record, GlobalAPI};

pub trait GokzRecord: Sized {
	fn replay_link(&self) -> Option<String>;
	fn view_link(&self) -> Option<String>;
	fn formatted_replay_links(tp: Option<&Self>, pro: Option<&Self>) -> Option<String>;
	fn formatted_view_links(tp: Option<&Self>, pro: Option<&Self>) -> Option<String>;
}

impl GokzRecord for Record {
	fn replay_link(&self) -> Option<String> {
		if self.replay_id != 0 {
			Some(GlobalAPI::get_replay_by_id(self.replay_id))
		} else {
			None
		}
	}

	fn view_link(&self) -> Option<String> {
		if self.replay_id != 0 {
			Some(format!(
				"http://gokzmaptest.site.nfoservers.com/GlobalReplays/?replay={}",
				self.replay_id
			))
		} else {
			None
		}
	}

	fn formatted_replay_links(tp: Option<&Self>, pro: Option<&Self>) -> Option<String> {
		let tp_link = match tp {
			Some(tp) => Self::replay_link(tp),
			None => None,
		};

		let pro_link = match pro {
			Some(pro) => Self::replay_link(pro),
			None => None,
		};

		match (tp_link, pro_link) {
			(Some(tp), Some(pro)) => Some(format!("Download Replays: [TP]({tp}) | [PRO]({pro})",)),
			(Some(tp), None) => Some(format!("Download Replay: [TP]({tp})")),
			(None, Some(pro)) => Some(format!("Download Replay: [PRO]({pro})")),
			(None, None) => None,
		}
	}

	fn formatted_view_links(tp: Option<&Self>, pro: Option<&Self>) -> Option<String> {
		let tp_link = match tp {
			Some(tp) => Self::view_link(tp),
			None => None,
		};

		let pro_link = match pro {
			Some(pro) => Self::view_link(pro),
			None => None,
		};

		match (tp_link, pro_link) {
			(Some(tp), Some(pro)) => Some(format!("Watch Replays: [TP]({tp}) | [PRO]({pro})",)),
			(Some(tp), None) => Some(format!("Watch Replay: [TP]({tp})")),
			(None, Some(pro)) => Some(format!("Watch Replay: [PRO]({pro})")),
			(None, None) => None,
		}
	}
}

pub fn fmt_time(time: f64) -> String {
	let seconds = time as u32;
	let hours = ((seconds / 3600) % 24) as u8;
	let seconds = seconds % 3600;
	let minutes = (seconds / 60) as u8;
	let seconds = seconds % 60;
	let millis = ((time - (time as u32) as f64) * 1000.0) as u16;

	let mut s = format!("{minutes:02}:{seconds:02}.{millis:03}");

	if hours > 0 {
		s = format!("{hours:02}:{s}");
	}

	s
}
