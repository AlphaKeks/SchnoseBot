//! Some extra utilities in addition to [`gokz_rs`] to make working with the `GlobalAPI` easier.

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

pub fn format_replay_links(
	tp_links: Option<(Option<String>, Option<String>)>,
	pro_links: Option<(Option<String>, Option<String>)>,
) -> Option<String> {
	let tp_links = match tp_links {
		None => None,
		Some(links) => {
			if let (Some(view_link), Some(download_link)) = links {
				Some((view_link, download_link))
			} else {
				None
			}
		}
	};

	let pro_links = match pro_links {
		None => None,
		Some(links) => {
			if let (Some(view_link), Some(download_link)) = links {
				Some((view_link, download_link))
			} else {
				None
			}
		}
	};

	match (tp_links, pro_links) {
		(Some((tp_view, tp_download)), Some((pro_view, pro_download))) => {
			Some(format!("TP Replay: [View Online]({tp_view}) | [Download]({tp_download})\nPRO Replay: [View Online]({pro_view}) | [Download]({pro_download})"))
		}
		(Some((tp_view, tp_download)), None) => {
			Some(format!("TP Replay: [View Online]({tp_view}) | [Download]({tp_download})"))
		}
		(None, Some((pro_view, pro_download))) => {
			Some(format!("PRO Replay: [View Online]({pro_view}) | [Download]({pro_download})"))
		}
		(None, None) => {
			None
		}
	}
}
