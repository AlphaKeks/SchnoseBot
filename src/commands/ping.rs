use {
	crate::events::slash_command::{InteractionData, InteractionResponseData::Message},
	anyhow::Result,
	serenity::builder::CreateApplicationCommand,
};

pub fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	return cmd.name("ping").description("pong!");
}

pub async fn execute(ctx: InteractionData<'_>) -> Result<()> {
	return ctx.reply(Message("pong!")).await;
}
