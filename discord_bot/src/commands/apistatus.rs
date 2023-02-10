use {
	crate::{error::Error, Context, State},
	gokz_rs::GlobalAPI,
	log::trace,
};

/// `GlobalAPI` health report.
///
/// Both this bot and GOKZ rely on the [GlobalAPI](https://portal.global-api.com/dashboard) to \
/// function properly. Sometimes it has downtimes though, and the bot commands might not work. You \
/// can either check the API's uptime via this command or on \
/// [this website](https://health.global-api.com/endpoints/_globalapi).
/// (The bot uses that website internally as well.)
#[poise::command(prefix_command, slash_command, on_error = "Error::handle_command")]
pub async fn apistatus(ctx: Context<'_>) -> Result<(), Error> {
	trace!("[/apistatus ({})]", ctx.author().tag());
	ctx.defer().await?;

	let health_report = GlobalAPI::checkhealth(ctx.gokz_client()).await?;

	let avg =
		(health_report.successful_responses as f64 + health_report.fast_responses as f64) / 2f64;
	let success = (avg * 10f64) as u8;

	let (status, color) = match success {
		90.. => ("Healthy", (116, 227, 161)),
		67.. => ("<:schnosesus:947467755727241287>", (249, 226, 175)),
		33.. => ("everything is on fire", (250, 179, 135)),
		_ => ("zer0.k wanted to be funny and pulled the usb stick again", (243, 139, 168)),
	};

	ctx.send(|reply| {
		reply.embed(|e| {
			e.color(color)
				.title(status)
				.url("https://health.global-api.com/endpoints/_globalapi")
				.thumbnail("https://dka575ofm4ao0.cloudfront.net/pages-transactional_logos/retina/74372/kz-icon.png")
				.field("Successful Healthchecks", format!("{} / {}", health_report.successful_responses, 10), true)
				.field("Fast Responses", format!("{} / {}", health_report.fast_responses, 10), true)
		})
	}).await?;

	Ok(())
}
