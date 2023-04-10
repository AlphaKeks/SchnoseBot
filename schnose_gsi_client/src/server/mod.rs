use {
	crate::gui::state::State,
	axum::{http::StatusCode, response::IntoResponse, routing::get, Json, Router, Server},
	gokz_rs::{MapIdentifier, Mode, SteamID},
	serde::{Deserialize, Serialize},
	std::{net::SocketAddr, sync::Arc},
	tokio::sync::Mutex,
};

mod gsi;
mod overlay;
mod pbs;
mod wrs;

pub const PORT: u16 = 9999;

#[derive(Debug, Clone)]
pub struct AxumState {
	state: Arc<Mutex<Option<State>>>,
	gokz_client: gokz_rs::Client,
}

pub async fn run(state: Arc<Mutex<Option<State>>>) {
	let axum_state = AxumState {
		state,
		gokz_client: gokz_rs::Client::new(),
	};

	let addr = SocketAddr::from(([127, 0, 0, 1], PORT));

	let router = Router::new()
		.route("/", get(overlay::handler))
		.route("/gsi", get(gsi::handler))
		.route("/pbs", get(pbs::handler))
		.route("/wrs", get(wrs::handler))
		.with_state(axum_state);

	Server::bind(&addr)
		.serve(router.into_make_service())
		.await
		.expect("Failed to run Axum server.");
}

#[derive(Debug, Clone, Serialize)]
struct Response {
	body: Option<State>,
}

impl IntoResponse for Response {
	fn into_response(self) -> axum::response::Response {
		match self.body {
			None => (StatusCode::NO_CONTENT, ()).into_response(),
			Some(json) => (StatusCode::OK, Json(json)).into_response(),
		}
	}
}

#[derive(Debug, Deserialize)]
pub struct GlobalAPIParams {
	pub steam_id: SteamID,
	pub map_identifier: MapIdentifier,
	pub mode: Mode,
}
