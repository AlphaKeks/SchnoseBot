use serenity::{
	builder::CreateApplicationCommand,
	model::prelude::interaction::application_command::CommandDataOption,
};

use crate::SchnoseCommand;

pub fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	cmd.name("ping").description("pong!")
}

pub fn run(_opts: &[CommandDataOption]) -> SchnoseCommand {
	SchnoseCommand::Message(String::from("pong!"))
}
