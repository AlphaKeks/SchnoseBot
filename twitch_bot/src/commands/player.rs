use {
	crate::{client::GlobalState, Result},
	gokz_rs::{
		schnose_api::{self, FancyPlayer},
		PlayerIdentifier,
	},
	tokio::time::{sleep, Duration},
};

#[tracing::instrument(skip(state))]
pub async fn execute(state: &GlobalState, player: PlayerIdentifier) -> Result<String> {
	let FancyPlayer { name, steam_id, is_banned: _, records } =
		schnose_api::get_player(player, &state.gokz_client).await?;

	let total_records = records.total;
	let kzt_tp = records.kzt.tp;
	let kzt_pro = records.kzt.pro;
	let skz_tp = records.skz.tp;
	let skz_pro = records.skz.pro;
	let vnl_tp = records.vnl.tp;
	let vnl_pro = records.vnl.pro;

	sleep(Duration::from_millis(727)).await;

	Ok(format!(
		"[{name} ({steam_id})] {total_records} Total Records | {kzt_tp} TP / {kzt_pro} PRO (KZT) | {skz_tp} TP / {skz_pro} PRO (SKZ) | {vnl_tp} TP / {vnl_pro} PRO (VNL)"
	))
}
