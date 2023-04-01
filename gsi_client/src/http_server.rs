use {
	axum::{
		extract::{Json, Query, State},
		http::StatusCode,
		response::IntoResponse,
		routing::{get, post},
		Router, Server,
	},
	gsi_client::gsi,
	serde::Serialize,
	std::{
		net::SocketAddr,
		sync::{Arc, Mutex},
	},
	tracing::{debug, info},
};

#[derive(Debug, Clone)]
pub struct InfoState(Arc<Mutex<Option<gsi::Info>>>);

impl From<Arc<Mutex<Option<gsi::Info>>>> for InfoState {
	fn from(value: Arc<Mutex<Option<gsi::Info>>>) -> Self {
		Self(value)
	}
}

pub async fn run(state: impl Into<InfoState>) {
	let addr = SocketAddr::from(([127, 0, 0, 1], 1337));

	let router = Router::new()
		.route("/", get(get_info))
		.route("/new_info", post(post_info))
		.with_state(state.into());

	info!("Listening on {addr}.");

	Server::bind(&addr)
		.serve(router.into_make_service())
		.await
		.expect("Failed to launch HTTP server.");
}

async fn post_info(
	State(InfoState(info_state)): State<InfoState>,
	Json(info): Json<gsi::Info>,
) -> impl IntoResponse {
	let Ok(mut lock) = info_state.lock() else {
		return StatusCode::INTERNAL_SERVER_ERROR;
	};

	*lock = Some(info);

	StatusCode::OK
}

#[derive(Debug)]
enum Response<T> {
	Success(Json<T>),
	Failure,
}

impl<T: Serialize> IntoResponse for Response<T> {
	fn into_response(self) -> axum::response::Response {
		match self {
			Response::Success(json) => (StatusCode::OK, json).into_response(),
			Response::Failure => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
		}
	}
}

async fn get_info(State(InfoState(info_state)): State<InfoState>) -> Response<gsi::Info> {
	let Ok(lock) = info_state.lock() else {
		return Response::Failure;
	};

	let Some(ref json) = *lock else {
		return Response::Failure;
	};

	Response::Success(Json(json.to_owned()))
}
