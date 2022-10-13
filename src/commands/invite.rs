use serenity::{
	builder::{CreateApplicationCommand, CreateEmbed},
	model::prelude::interaction::application_command::CommandDataOption,
};

use crate::SchnoseCommand;

pub fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	cmd.name("invite")
		.description("Invite schnose to your server")
}

pub fn run(_opts: &[CommandDataOption]) -> SchnoseCommand {
	let embed = CreateEmbed::default().color((116, 128, 194)).description("[Click me!](https://discord.com/oauth2/authorize?client_id=940308056451973120&permissions=327744&scope=bot%20applications.commands)").to_owned();

	SchnoseCommand::Embed(embed)
}
