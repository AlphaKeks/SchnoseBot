use {
	crate::{client::GlobalState, Result},
	gokz_rs::global_api,
};

pub async fn execute(state: &GlobalState) -> Result<String> {
	let status = global_api::checkhealth(&state.gokz_client).await?;

	let avg = (status.successful_responses as f64 + status.fast_responses as f64) / 2f64;
	let success = (avg * 10f64) as u8;

	let message = match success {
		90.. => "Healthy",
		67.. => "Susge",
		33.. => "monkaS",
		_ => "Deadge",
	};

	Ok(format!(
		"{} - {}/{} Successful Responses - {}/{} Fast Responses",
		message, status.successful_responses, 10, status.fast_responses, 10
	))
}
