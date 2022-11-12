use {
	crate::events::slash_command::{InteractionData, InteractionResponseData::Embed},
	anyhow::Result,
	serenity::builder::{CreateApplicationCommand, CreateEmbed},
};

pub fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	return cmd.name("invite").description("Invite schnose to your server!");
}

pub async fn execute(ctx: InteractionData<'_>) -> Result<()> {
	let embed = CreateEmbed::default()
		.color((116, 128, 194))
		.description("[click? 😳 👉👈](https://discord.com/oauth2/authorize?client_id=940308056451973120&permissions=327744&scope=bot%20applications.commands)")
		.to_owned();

	return ctx.reply(Embed(embed)).await;
}
