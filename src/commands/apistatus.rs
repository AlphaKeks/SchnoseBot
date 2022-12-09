use {
	crate::{events::interactions::InteractionState, prelude::InteractionResult},
	gokz_rs::global_api::health_check,
	serenity::builder::{CreateApplicationCommand, CreateEmbed},
};

pub(crate) fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	return cmd.name("apistatus").description("Check the GlobalAPI's health status.");
}

pub(crate) async fn execute(state: &mut InteractionState<'_>) -> InteractionResult {
	// Defer current interaction since this could take a while
	state.defer().await?;

	let response = health_check(&state.req_client).await?;

	let (success, mut status, mut color) = (
		(response.successful_responses + response.fast_responses) as f32 / 2f32,
		"Healthy",
		(166, 227, 161),
	);

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

	let embed = CreateEmbed::default()
		.color(color)
		.title(status)
		.url("https://health.global-api.com/endpoints/_globalapi")
		.thumbnail("https://dka575ofm4ao0.cloudfront.net/pages-transactional_logos/retina/74372/kz-icon.png")
		.field("Successful Healthchecks", format!("{} / {}", response.successful_responses, 10), true)
		.field("Fast Responses", format!("{} / {}", response.fast_responses, 10), true)
		.to_owned();

	return Ok(embed.into());
}
