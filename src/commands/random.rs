use gokz_rs::global_api::get_maps;
use rand::Rng;
use serenity::{builder::CreateApplicationCommand, model::prelude::command::CommandOptionType};

use crate::event_handler::interaction_create::{Metadata, SchnoseResponseData};

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

pub async fn run(metadata: Metadata) {
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
			return metadata.reply(SchnoseResponseData::Message(why.tldr)).await;
		},
		Ok(maps) => match metadata.opts.get_int("tier") {
			Some(tier) => maps
				.into_iter()
				.filter(|map| map.difficulty == (tier as u8))
				.collect::<Vec<_>>(),
			None => maps,
		},
	};

	let rng = rand::thread_rng().gen_range(0..global_maps.len());
	return metadata
		.reply(SchnoseResponseData::Message(format!(
			"🎲 {} (T{})",
			global_maps[rng].name, global_maps[rng].difficulty
		)))
		.await;
}
