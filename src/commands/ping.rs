use {
	crate::events::slash_commands::{GlobalState, InteractionResponseData::Message},
	serenity::builder::CreateApplicationCommand,
};

pub(crate) fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	return cmd.name("ping").description("pong!");
}

pub(crate) async fn execute(state: GlobalState<'_>) -> anyhow::Result<()> {
	return state.reply(Message("pong!")).await;
}
