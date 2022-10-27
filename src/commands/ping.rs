use serenity::builder::CreateApplicationCommand;

use crate::event_handler::interaction_create::SchnoseResponseData;

pub fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	cmd.name("ping").description("pong!")
}

pub fn run() -> SchnoseResponseData {
	return SchnoseResponseData::Message(String::from("pong!"));
}
