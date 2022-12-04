use gokz_rs::global_api;

type PB = Result<gokz_rs::global_api::records::top::Record, gokz_rs::prelude::Error>;
pub(crate) fn get_player_name(records: (&PB, &PB)) -> String {
	match records.0 {
		Ok(tp) => tp.player_name.clone().unwrap_or(String::from("unknown")),
		Err(_) => match records.1 {
			Ok(pro) => pro.player_name.clone().unwrap_or(String::from("unknown")),
			Err(_) => String::from("unknown"),
		},
	}
}

pub(crate) async fn get_place(record: &PB, client: &reqwest::Client) -> String {
	if let Ok(record) = record {
		if let Ok(place) = global_api::get_place(&record.id, client).await {
			return format!("[#{}]", place.0);
		}
	}
	return String::new();
}

pub(crate) async fn get_replay_link(record: &PB) -> String {
	if let Ok(record) = record {
		if record.replay_id != 0 {
			if let Ok(link) = global_api::get_replay(record.replay_id).await {
				return link;
			}
		}
	}

	return String::new();
}
