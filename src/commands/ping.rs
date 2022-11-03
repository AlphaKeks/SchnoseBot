use serenity::builder::CreateApplicationCommand;

use crate::event_handler::interaction_create::{Metadata, SchnoseResponseData};

pub fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	cmd.name("ping").description("pong!")
}

pub async fn run(metadata: Metadata) {
	let pong = SchnoseResponseData::Message(String::from("pong!"));
	metadata.reply(pong).await
}
