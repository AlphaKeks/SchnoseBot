use gokz_rs::functions::get_maps;
use rand::seq::SliceRandom;
use serenity::{
	builder::CreateApplicationCommand,
	model::prelude::{
		command::CommandOptionType, interaction::application_command::CommandDataOption,
	},
};

use crate::{util::get_integer, SchnoseCommand};

pub fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	cmd.name("random")
		.description("Get a random KZ map")
		.create_option(|opt| {
			opt.kind(CommandOptionType::Integer)
				.name("tier")
				.description("Filter by tier")
				.add_int_choice("1 (Very Easy)", 1)
				.add_int_choice("2 (Easy)", 2)
				.add_int_choice("3 (Medium)", 3)
				.add_int_choice("4 (Hard)", 4)
				.add_int_choice("5 (Very Hard)", 5)
				.add_int_choice("6 (Extreme)", 6)
				.add_int_choice("7 (Death)", 7)
				.required(false)
		})
}

pub async fn run(opts: &[CommandDataOption]) -> SchnoseCommand {
	let client = reqwest::Client::new();

	let mut global_maps = match get_maps(&client).await {
		Ok(maps) => maps,
		Err(why) => return SchnoseCommand::Message(why.tldr),
	};

	if let Some(tier) = get_integer("tier", opts) {
		global_maps = global_maps
			.into_iter()
			.filter(|m| m.difficulty == tier as u8)
			.collect::<Vec<gokz_rs::global_api::maps::Response>>();
	}

	match global_maps.choose(&mut rand::thread_rng()) {
		Some(result) => {
			return SchnoseCommand::Message(format!("🎲 {} (T{})", result.name, result.difficulty))
		}
		None => return SchnoseCommand::Message(String::from("Failed to select random map.")),
	}
}
