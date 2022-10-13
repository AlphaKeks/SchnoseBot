use gokz_rs::functions::check_api;
use serenity::{
	builder::{CreateApplicationCommand, CreateEmbed},
	model::prelude::interaction::application_command::CommandDataOption,
};

use crate::SchnoseCommand;

pub fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	cmd.name("apistatus")
		.description("Check the current status of the GlobalAPI")
}

pub async fn run(_opts: &[CommandDataOption]) -> SchnoseCommand {
	match check_api(&reqwest::Client::new()).await {
		Ok(data) => {
			let color = if data.frontend == "operational" && data.backend == "operational" {
				(166, 227, 161)
			} else if data.frontend == "operational" || data.backend == "operational" {
				(250, 179, 135)
			} else {
				(243, 139, 168)
			};

			let embed = CreateEmbed::default()
				.title(data.status)
				.url("https://status.global-api.com/")
				.thumbnail("https://dka575ofm4ao0.cloudfront.net/pages-transactional_logos/retina/74372/kz-icon.png")
				.color(color)
				.field("frontend", data.frontend, true)
				.field("backend", data.backend, true)
				.to_owned();

			return SchnoseCommand::Embed(embed);
		}
		Err(why) => return SchnoseCommand::Message(why.tldr),
	};
}
