use {
	crate::events::slash_commands::{InteractionData, InteractionResponseData::Message},
	serenity::builder::CreateApplicationCommand,
};

pub(crate) fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	return cmd.name("ping").description("pong!");
}

pub(crate) async fn execute(data: InteractionData<'_>) -> anyhow::Result<()> {
	return data.reply(Message("pong!")).await;
}
