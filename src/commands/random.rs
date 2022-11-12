use rand::Rng;

use {
	crate::events::slash_command::{InteractionData, InteractionResponseData::Message},
	anyhow::Result,
	gokz_rs::global_api::*,
	serenity::{builder::CreateApplicationCommand, model::prelude::command::CommandOptionType},
};

pub fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	return cmd.name("random").description("Get a random KZ map.").create_option(|opt| {
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
	});
}

pub async fn execute(mut ctx: InteractionData<'_>) -> Result<()> {
	ctx.defer().await?;

	let global_maps = match get_maps(&reqwest::Client::new()).await {
		Ok(maps) => match ctx.get_int("tier") {
			Some(tier) => maps
				.into_iter()
				.filter(|map| map.difficulty == (tier as u8))
				.collect::<Vec<_>>(),
			None => maps,
		},
		Err(why) => {
			log::error!(
				"[{}]: {} => {}\n{:?}",
				file!(),
				line!(),
				"Failed to fetch global maps.",
				why
			);
			return ctx.reply(Message(&why.tldr)).await;
		},
	};

	let rand = rand::thread_rng().gen_range(0..global_maps.len());

	return ctx
		.reply(Message(&format!(
			"🎲 {} (T{})",
			global_maps[rand].name, global_maps[rand].difficulty
		)))
		.await;
}
