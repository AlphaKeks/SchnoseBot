use {
	axum::{
		extract::State as AxumState,
		http::{HeaderMap, StatusCode},
		response::{Html, IntoResponse},
		routing::get,
		Json, Router, Server,
	},
	gokz_rs::{global_api::Record, Mode, SteamID, Tier},
	serde::{Serialize, Serializer},
	std::{
		net::SocketAddr,
		sync::{Arc, Mutex},
	},
	tokio::sync::mpsc::UnboundedReceiver,
	tracing::{debug, error},
};

fn ser_mode<S>(mode: &Option<Mode>, serializer: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	mode.map(|mode| mode.short())
		.serialize(serializer)
}

#[derive(Debug, Clone, Serialize)]
pub struct Payload {
	pub map_name: String,
	pub map_tier: Option<Tier>,
	#[serde(serialize_with = "ser_mode")]
	pub mode: Option<Mode>,
	pub steam_id: Option<SteamID>,
	pub tp_wr: Option<Record>,
	pub pro_wr: Option<Record>,
	pub tp_pb: Option<Record>,
	pub pro_pb: Option<Record>,
}

#[derive(Debug, Clone)]
struct State {
	current_payload: Arc<Mutex<Payload>>,
	receiver: Arc<Mutex<UnboundedReceiver<Payload>>>,
}

pub async fn run(mut receiver: UnboundedReceiver<Payload>) {
	let initial_payload = receiver
		.recv()
		.await
		.expect("Channel has been closed earlier than expected.");

	let state = State {
		current_payload: Arc::new(Mutex::new(initial_payload)),
		receiver: Arc::new(Mutex::new(receiver)),
	};

	let addr = SocketAddr::from(([127, 0, 0, 1], 9999));
	let router = Router::new()
		.route("/", get(overlay))
		.route("/gsi", get(recv))
		.with_state(state);

	Server::bind(&addr)
		.serve(router.into_make_service())
		.await
		.expect("Failed to run Axum server.");
}

#[derive(Debug, Clone, Serialize)]
struct Response(Option<Payload>);

impl IntoResponse for Response {
	fn into_response(self) -> axum::response::Response {
		match self.0 {
			None => ().into_response(),
			Some(json) => Json(json).into_response(),
		}
	}
}

async fn overlay() -> impl IntoResponse {
	Html(include_str!("../static/overlay.html"))
}

async fn recv(AxumState(state): AxumState<State>, headers: HeaderMap) -> impl IntoResponse {
	debug!("Headers: {headers:?}");

	let mut current_payload = match state.current_payload.lock() {
		Ok(guard) => guard,
		Err(why) => {
			error!("Failed to acquire payload Mutex: {why:?}");
			return (StatusCode::INTERNAL_SERVER_ERROR, Response(None));
		}
	};

	let mut receiver = match state.receiver.lock() {
		Ok(guard) => guard,
		Err(why) => {
			error!("Failed to acquire receiver Mutex: {why:?}");
			return (StatusCode::INTERNAL_SERVER_ERROR, Response(None));
		}
	};

	let payload = match receiver.try_recv() {
		Ok(new_payload) => {
			*current_payload = new_payload.clone();
			new_payload
		}
		Err(why) => {
			error!("No new data? {why:?}");
			(*current_payload).clone()
		}
	};

	(StatusCode::OK, Response(Some(payload)))
}
