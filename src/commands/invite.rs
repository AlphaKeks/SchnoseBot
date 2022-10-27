use serenity::builder::{CreateApplicationCommand, CreateEmbed};

use crate::event_handler::interaction_create::SchnoseResponseData;

pub fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	cmd.name("invite").description("Invite schnose to your server!")
}

pub fn run() -> SchnoseResponseData {
	let embed = CreateEmbed::default()
		.color((116, 128, 194))
		.description("[click? 😳 👉👈](https://discord.com/oauth2/authorize?client_id=940308056451973120&permissions=327744&scope=bot%20applications.commands)")
		.to_owned();

	return SchnoseResponseData::Embed(embed);
}
