use {
	super::{AxumState, GlobalAPIParams},
	axum::{
		extract::{Json, Query, State as StateExtractor},
		response::IntoResponse,
	},
	gokz_rs::global_api,
};

pub async fn handler(
	Query(GlobalAPIParams { map_identifier, mode, .. }): Query<GlobalAPIParams>,
	StateExtractor(AxumState { gokz_client, .. }): StateExtractor<AxumState>,
) -> impl IntoResponse {
	let tp = global_api::get_wr(map_identifier.clone(), mode, true, 0, &gokz_client)
		.await
		.ok();

	let pro = global_api::get_wr(map_identifier.clone(), mode, false, 0, &gokz_client)
		.await
		.ok();

	Json((tp, pro))
}
