use {
	axum::{
		extract::{Json, State},
		http::{HeaderMap, StatusCode},
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
	tracing::{debug, error, info},
};

#[derive(Debug, Clone)]
pub struct InfoState {
	info_state: Arc<Mutex<Option<gsi::Info>>>,
	gokz_client: gokz_rs::BlockingClient,
}

impl From<Arc<Mutex<Option<gsi::Info>>>> for InfoState {
	fn from(value: Arc<Mutex<Option<gsi::Info>>>) -> Self {
		Self {
			info_state: value,
			gokz_client: gokz_rs::BlockingClient::new(),
		}
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
	headers: HeaderMap,
	State(InfoState { info_state, gokz_client }): State<InfoState>,
	Json(info): Json<gsi::Info>,
) -> impl IntoResponse {
	let Some(api_key) = headers.get("x-schnose-auth-key") else {
		return StatusCode::BAD_REQUEST;
	};

	let Ok(api_key) = api_key.to_str() else {
		return StatusCode::BAD_REQUEST;
	};

	let request = gokz_client
		.post("http://schnose.xyz/api/twitch_info")
		.header("x-schnose-auth-key", api_key)
		.json(&info);

	let Ok(mut lock) = info_state.lock() else {
		return StatusCode::INTERNAL_SERVER_ERROR;
	};

	*lock = Some(info);

	debug!("POSTing to SchnoseAPI...");
	match request
		.send()
		.map(|res| res.error_for_status())
	{
		Ok(Ok(res)) => info!("POST to SchnoseAPI was successful: {res:#?}"),
		Ok(Err(why)) | Err(why) => error!("POST to SchnoseAPI failed: {why:#?}"),
	}

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

async fn get_info(State(InfoState { info_state, .. }): State<InfoState>) -> Response<gsi::Info> {
	let Ok(lock) = info_state.lock() else {
		return Response::Failure;
	};

	let Some(ref json) = *lock else {
		return Response::Failure;
	};

	Response::Success(Json(json.to_owned()))
}
