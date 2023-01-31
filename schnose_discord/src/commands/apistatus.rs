use {
	super::handle_err,
	crate::{GlobalStateAccess, SchnoseError},
	gokz_rs::GlobalAPI,
	log::trace,
};

/// Check the GlobalAPI's current health status.
#[poise::command(prefix_command, slash_command, on_error = "handle_err")]
pub async fn apistatus(ctx: crate::Context<'_>) -> Result<(), SchnoseError> {
	// delay sending the message
	ctx.defer().await?;

	trace!("[/apistatus] ({})", &ctx.author().tag());

	let health = GlobalAPI::checkhealth(ctx.gokz_client()).await?;

	let success =
		((f32::from(health.successful_responses + health.fast_responses) / 2f32) * 10.0) as u8;

	let (status, color) = match success {
		90.. => ("Healthy", (116, 227, 161)),
		67.. => ("<:schnosesus:947467755727241287>", (249, 226, 175)),
		33.. => ("everything is on fire", (250, 179, 135)),
		_ => ("zer0.k wanted to be funny and pulled the usb stick again 😂", (243, 139, 168)),
	};

	ctx.send(|reply| {
		reply.embed(|e| {
			e.color(color)
			.title(status)
				.url("https://health.global-api.com/endpoints/_globalapi")
				.thumbnail("https://dka575ofm4ao0.cloudfront.net/pages-transactional_logos/retina/74372/kz-icon.png")
				.field("Successful Healthchecks", format!("{} / {}", health.successful_responses, 10), true)
				.field("Fast Responses", format!("{} / {}", health.fast_responses, 10), true)
		})
	}).await?;

	Ok(())
}
