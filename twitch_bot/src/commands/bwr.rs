use {
	crate::{client::GlobalState, global_maps::GlobalMap, util::fmt_time, Result},
	gokz_rs::{global_api, Mode},
};

pub async fn execute(
	state: &GlobalState,
	map: GlobalMap,
	mode: Mode,
	course: u8,
) -> Result<String> {
	let tp = global_api::get_wr(map.id.into(), mode, true, course, &state.gokz_client).await;
	let pro = global_api::get_wr(map.id.into(), mode, false, course, &state.gokz_client).await;

	let map = map.name;
	let mode = mode.short();

	let tp = if let Ok(record) = tp {
		format!(
			"{} ({}) by {}",
			fmt_time(record.time),
			match record.teleports {
				1 => String::from("1 TP"),
				n => format!("{n} TPs"),
			},
			record.player_name
		)
	} else {
		String::from("no TP record")
	};

	let pro = if let Ok(record) = pro {
		format!("{} by {}", fmt_time(record.time), record.player_name)
	} else {
		String::from("no PRO record")
	};

	Ok(format!("[BWR {course} on {map} in {mode}] TP: {tp} / PRO: {pro}"))
}
