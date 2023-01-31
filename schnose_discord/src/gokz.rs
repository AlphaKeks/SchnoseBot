use gokz_rs::{records::Record, GlobalAPI};

pub trait ExtractRecordInfo {
	fn get_player_name(&self) -> String;
	fn get_replay_links(&self) -> (String, String);
}

type PotentialRecord = Result<Record, gokz_rs::prelude::Error>;

impl ExtractRecordInfo for (&PotentialRecord, &PotentialRecord) {
	fn get_player_name(&self) -> String {
		if let Ok(tp) = self.0 {
			if let Some(ref name) = tp.player_name {
				return name.to_owned();
			}
		}

		if let Ok(pro) = self.1 {
			if let Some(ref name) = pro.player_name {
				return name.to_owned();
			}
		}

		String::from("unknown")
	}

	fn get_replay_links(&self) -> (String, String) {
		let (mut view_link, mut download_link) = (String::new(), String::new());

		let watch_link = "http://gokzmaptest.site.nfoservers.com/GlobalReplays/?replay=";

		match self {
			(Ok(tp), Ok(pro)) if tp.replay_id != 0 && pro.replay_id != 0 => {
				view_link = format!(
					"Watch Replays: [TP]({}{}) | [PRO]({}{})",
					watch_link, tp.replay_id, watch_link, pro.replay_id
				);

				download_link = format!(
					"Download Replays: [TP]({}) | [PRO]({})",
					GlobalAPI::get_replay_by_id(tp.replay_id),
					GlobalAPI::get_replay_by_id(pro.replay_id),
				);
			}
			(Ok(tp), _) if tp.replay_id != 0 => {
				view_link = format!("Watch Replay: [TP]({}{})", watch_link, tp.replay_id);

				download_link =
					format!("Download Replay: [TP]({})", GlobalAPI::get_replay_by_id(tp.replay_id));
			}
			(_, Ok(pro)) if pro.replay_id != 0 => {
				view_link = format!("Watch Replay: [PRO]({}{})", watch_link, pro.replay_id);

				download_link = format!(
					"Download Replay: [PRO]({})",
					GlobalAPI::get_replay_by_id(pro.replay_id)
				);
			}
			_ => {}
		}

		(view_link, download_link)
	}
}
