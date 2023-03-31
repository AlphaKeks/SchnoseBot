use {
	crate::{client::GlobalState, global_maps::GlobalMap, util::fmt_time, Result},
	gokz_rs::{global_api, Mode, PlayerIdentifier},
	tokio::time::{sleep, Duration},
};

#[tracing::instrument(skip(state))]
pub async fn execute(
	state: &GlobalState,
	map: GlobalMap,
	player: PlayerIdentifier,
	mode: Mode,
	course: u8,
) -> Result<String> {
	let tp =
		global_api::get_pb(player.clone(), map.id.into(), mode, true, course, &state.gokz_client)
			.await;
	let pro =
		global_api::get_pb(player.clone(), map.id.into(), mode, false, course, &state.gokz_client)
			.await;

	let map = map.name;
	let mode = mode.short();
	let mut player_name = player;

	let tp = if let Ok(record) = tp {
		player_name = record.player_name.into();
		format!(
			"{} ({})",
			fmt_time(record.time),
			match record.teleports {
				1 => String::from("1 TP"),
				n => format!("{n} TPs"),
			}
		)
	} else {
		String::from("no TP record")
	};

	let pro = if let Ok(record) = pro {
		player_name = record.player_name.into();
		fmt_time(record.time)
	} else {
		String::from("no PRO record")
	};

	sleep(Duration::from_millis(727)).await;

	Ok(format!("[{player_name} on {map} B{course} in {mode}] TP: {tp} / PRO: {pro}"))
}
