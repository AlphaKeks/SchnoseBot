use gokz_rs::global_api::get_maps;
use rand::seq::SliceRandom;
use serenity::{builder::CreateApplicationCommand, model::prelude::command::CommandOptionType};

use crate::event_handler::interaction_create::{CommandOptions, SchnoseResponseData};

pub fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	cmd.name("random").description("Get a random KZ map.").create_option(|opt| {
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

pub async fn run<'a>(opts: CommandOptions<'a>) -> SchnoseResponseData {
	let client = reqwest::Client::new();

	let global_maps = match get_maps(&client).await {
		Err(why) => {
			log::error!(
				"[{}]: {} => {}\n{:#?}",
				file!(),
				line!(),
				"Failed to get global maps.",
				why
			);

			return SchnoseResponseData::Message(why.tldr);
		},
		Ok(maps) => match opts.get_int("tier") {
			Some(tier) => maps
				.into_iter()
				.filter(|map| map.difficulty == (tier as u8))
				.collect::<Vec<_>>(),
			None => maps,
		},
	};

	match global_maps.choose(&mut rand::thread_rng()) {
		Some(result) => {
			return SchnoseResponseData::Message(format!(
				"🎲 {} (T{})",
				result.name, result.difficulty
			))
		},
		None => {
			log::error!("[{}]: {} => {}", file!(), line!(), "Failed to select random map.",);

			return SchnoseResponseData::Message(String::from("Failed to select random map."));
		},
	}
}
