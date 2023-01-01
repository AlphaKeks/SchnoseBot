use {
	super::handle_err,
	crate::{GlobalStateAccess, SchnoseError},
	log::trace,
	gokz_rs::GlobalAPI,
};

/// Check the GlobalAPI's current health status.
#[poise::command(prefix_command, slash_command, on_error = "handle_err")]
pub async fn apistatus(ctx: crate::Context<'_>) -> Result<(), SchnoseError> {
	// delay sending the message
	ctx.defer().await?;

	trace!("[/apistatus] ({})", &ctx.author().tag());

	let health = GlobalAPI::checkhealth(ctx.gokz_client()).await?;

	let success = f32::from(health.successful_responses + health.fast_responses) / 2f32;
	let mut status = "Healthy";
	let mut color = (116, 227, 161);

	if success < 9.0 {
		status = "<:schnosesus:947467755727241287>";
		color = (249, 226, 175);
	}

	if success < 6.7 {
		status = "everything is on fire";
		color = (250, 179, 135);
	}

	if success < 3.3 {
		status = "zer0.k wanted to be funny and pulled the usb stick again 😂";
		color = (243, 139, 168);
	}

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
