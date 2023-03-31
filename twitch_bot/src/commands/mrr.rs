use {
	crate::{client::GlobalState, Result},
	gokz_rs::schnose_api,
	schnosebot::formatting::fmt_time,
	tokio::time::{sleep, Duration},
};

#[tracing::instrument(skip(state))]
pub async fn execute(state: &GlobalState) -> Result<String> {
	let recent = schnose_api::get_records(1, &state.gokz_client)
		.await?
		.remove(0);

	let player_name = recent.player.name;
	let map = recent.map_name;
	let mode = recent.mode.short();
	let (runtype, teleports) = match recent.teleports {
		0 => ("PRO", String::new()),
		1 => ("TP", String::from("(1 TP)")),
		n => ("TP", format!("({n} TPs)")),
	};
	let time = fmt_time(recent.time);
	let date = recent
		.created_on
		.format("%d-%m-%Y %H:%M:%S");

	sleep(Duration::from_millis(727)).await;

	Ok(format!("[{player_name} on {map} in {mode} {runtype}] {time} {teleports} on {date}"))
}
