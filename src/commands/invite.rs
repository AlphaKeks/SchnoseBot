use {
	crate::{prelude::InteractionResult, events::interactions::InteractionState},
	serenity::builder::{CreateApplicationCommand, CreateEmbed},
};

pub(crate) fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	return cmd.name("invite").description("Invite schnose to your server!");
}

pub(crate) async fn execute(state: &mut InteractionState<'_>) -> InteractionResult {
	let embed = CreateEmbed::default()
		.color((116, 128, 194))
		.description("[click? 😳 👉👈](https://discord.com/oauth2/authorize?client_id=940308056451973120&permissions=327744&scope=bot%20applications.commands)")
		.to_owned();

	state.ephemeralize();
	Ok(embed.into())
}
