use std::collections::HashMap;

use gokz_rs::get_map;
use serenity::{
	builder::{CreateApplicationCommand, CreateEmbed},
	json,
	model::prelude::{
		command::CommandOptionType, interaction::application_command::CommandDataOption,
	},
};

use crate::SchnoseCommand;

pub fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	cmd.name("map")
		.description("balls map")
		.create_option(|opt| {
			opt.name("map")
				.description("the map you want to look up")
				.kind(CommandOptionType::String)
				.required(true)
		})
}

pub async fn run(
	opts: &[CommandDataOption],
	// data: &'a mut CreateInteractionResponseData<'a>,
) -> SchnoseCommand {
	let mut embed_data = HashMap::new();

	let input_map = match &opts[0].value {
		Some(string) => match string.to_owned() {
			json::Value::String(str) => Some(str),
			_ => None,
		},
		None => return SchnoseCommand::Message(String::from("Please provide a valid map name.")),
	};

	match input_map {
		None => return SchnoseCommand::Message(String::from("Please provide a valid map name.")),
		Some(map_name) => {
			let client = reqwest::Client::new();

			let map = match get_map(
				gokz_rs::global_api::GOKZMapIdentifier::Name(map_name),
				&client,
			)
			.await
			{
				Ok(data) => data,
				Err(why) => return SchnoseCommand::Message(String::from(why.tldr)),
			};

			embed_data.insert("title", json::Value::String(map.name));
			embed_data.insert(
				"description",
				json::Value::String(format!(
					"created at: {}\nupdated at: {}",
					map.created_on, map.updated_on
				)),
			);

			return SchnoseCommand::Embed(CreateEmbed(embed_data));
		}
	}
}
