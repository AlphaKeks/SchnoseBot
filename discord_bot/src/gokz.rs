//! Some extra utilities in addition to [`gokz_rs`] to make working with the `GlobalAPI` easier.

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
