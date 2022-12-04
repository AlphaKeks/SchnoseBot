use {
	crate::events::slash_commands::InteractionResponseData::{self, *},
	serenity::builder::CreateApplicationCommand,
};

pub(crate) fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	return cmd.name("ping").description("pong!");
}

pub(crate) async fn execute() -> anyhow::Result<InteractionResponseData> {
	return Ok(Message("pong!".into()));
}
