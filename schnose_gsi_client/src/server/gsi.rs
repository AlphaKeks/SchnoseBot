use {
	super::{AxumState, Response},
	axum::{extract::State as StateExtractor, response::IntoResponse},
};

pub async fn handler(
	StateExtractor(AxumState { state, .. }): StateExtractor<AxumState>,
) -> impl IntoResponse {
	Response { body: state.lock().await.clone() }
}
