use gokz_rs::global_api::health_check;
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
	match health_check(&reqwest::Client::new()).await {
		Ok(data) => {
			let success = (data.successful_responses + data.fast_responses) as f32 / 2.0;
			let (mut status, mut color) = ("we good", (166, 227, 161));

			if success < 9.0 {
				status = "<:schnosesus:947467755727241287>";
				color = (249, 226, 175);
			}

			if success < 6.6 {
				status = "everything is on fire";
				color = (250, 179, 135);
			}

			if success < 3.3 {
				status = "gc wanted to be funny and pulled the usb stick again";
				color = (243, 139, 168);
			}

			let embed = CreateEmbed::default()
				.title(status)
				.url("https://health.global-api.com/endpoints/_globalapi")
				.thumbnail("https://dka575ofm4ao0.cloudfront.net/pages-transactional_logos/retina/74372/kz-icon.png")
				.color(color)
				.field(
					"Successful healthchecks",
					format!("{} / {}", data.successful_responses, 10),
					true,
				)
				.field(
					"Fast responses",
					format!("{} / {}", data.fast_responses, 10),
					true,
				)
				.to_owned();

			return SchnoseCommand::Embed(embed);
		}
		Err(why) => {
			log::error!("`check_api`: {:#?}", why);

			return SchnoseCommand::Message(why.tldr);
		}
	};
}
