use {
	crate::{events::slash_commands::InteractionResponseData::*, schnose::InteractionResult},
	serenity::builder::CreateApplicationCommand,
};

pub(crate) fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	return cmd.name("ping").description("pong!");
}

pub(crate) async fn execute() -> InteractionResult {
	return Ok(Message("pong!".into()));
}
