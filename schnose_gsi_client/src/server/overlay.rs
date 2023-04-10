use {
	axum::response::{Html, IntoResponse},
	std::path::PathBuf,
};

pub async fn handler() -> impl IntoResponse {
	if let Ok(reload_path) = std::env::var("RELOAD_GSI_OVERLAY") {
		let path = PathBuf::from(reload_path);
		return Html(
			tokio::fs::read_to_string(path)
				.await
				.expect("Failed to read HTML"),
		);
	}

	Html(include_str!("../../assets/overlay/index.html").to_owned())
}
