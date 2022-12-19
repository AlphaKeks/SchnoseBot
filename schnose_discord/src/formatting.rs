use {std::fmt::Write, gokz_rs::global_api, serenity::builder::CreateEmbed};

/// Turns an amount of seconds into a nicely formatted string:
/// ```
/// let time: f32 = 1263.7832;
/// assert_eq!("00:21:03.78", &format_time(time));
/// ```
pub(crate) fn format_time(secs_float: f32) -> String {
	let seconds = secs_float as u32;
	let hours = ((seconds / 3600) % 24) as u8;
	let seconds = seconds % 3600;
	let minutes = (seconds / 60) as u8;
	let seconds = seconds % 60;
	let millis = ((secs_float - (secs_float as u32) as f32) * 1000.0) as u16;

	let mut s = String::new();

	let _ = write!(&mut s, "{:02}:{:02}.{:03}", minutes, seconds, millis);

	if hours > 0 {
		s = format!("{:02}:{}", hours, s);
	}

	s
}

/// Takes an embed and attaches properly formatted "Download Replays" links.
pub(crate) fn attach_replay_links(
	embed: &mut CreateEmbed,
	links: ((String, String), (String, String)),
) -> &mut CreateEmbed {
	let tp = links.0 .0.is_empty();
	let pro = links.1 .0.is_empty();

	if tp || pro {
		let text = if tp && !pro {
			format!(
				"Watch Replays:            [TP]({})\nDownload Replays:     [TP]({})",
				links.0 .0, links.0 .1
			)
		} else if !tp && pro {
			format!(
				"Watch Replays:            [PRO]({})\nDownload Replays:     [PRO]({})",
				links.1 .0, links.1 .1
			)
		} else {
			format!(
				"Watch Replays:            [TP]({}) | [PRO]({})\nDownload Replays:     [TP]({}) | [PRO]({})",
				links.0 .0, links.1 .0, links.1 .0, links.1 .1
			)
		};

		embed.description(text);
		return embed;
	}

	embed
}

type PB = Result<gokz_rs::global_api::records::top::Record, gokz_rs::prelude::Error>;

/// Utility function to extract a player's name from a set of records. Usually used in commands
/// where there are 2 runs by the same player, e.g. `/pb` or similar.
pub(crate) fn get_player_name(records: (&PB, &PB)) -> String {
	match records.0 {
		Ok(tp) => tp.player_name.clone().unwrap_or_else(|| String::from("unknown")),
		Err(_) => match records.1 {
			Ok(pro) => pro.player_name.clone().unwrap_or_else(|| String::from("unknown")),
			Err(_) => String::from("unknown"),
		},
	}
}

/// Utility function to fetch a record's place and return either a nicely formatted, or an empty
/// String
pub(crate) async fn get_place_formatted(record: &PB, client: &gokz_rs::Client) -> String {
	if let Ok(record) = record {
		if let Ok(place) = global_api::get_place(&record.id, client).await {
			return format!("[#{}]", place.0);
		}
	}
	String::new()
}

/// Utility function to generate a replay download link
pub(crate) async fn get_replay_links(record: &PB) -> (String, String) {
	if let Ok(record) = record {
		if record.replay_id != 0 {
			if let Ok(link) = global_api::get_replay(record.replay_id).await {
				return (
					format!(
						"http://gokzmaptest.site.nfoservers.com/GlobalReplays/?replay={}",
						&record.replay_id
					),
					link,
				);
			}
		}
	}
	(String::new(), String::new())
}
