use rand::Rng;

use {
	crate::events::slash_commands::{InteractionData, InteractionResponseData::Message},
	gokz_rs::global_api::*,
	serenity::{builder::CreateApplicationCommand, model::prelude::command::CommandOptionType},
};

pub(crate) fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
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

pub(crate) async fn execute(data: InteractionData<'_>) -> anyhow::Result<()> {
	let tier = match data.get_int("tier") {
		Some(tier) => Some(tier as u8),
		None => None,
	};
	let map_names = match get_mapcycle(tier, &data.req_client).await {
		Ok(names) => names,
		Err(why) => {
			log::warn!("[{}]: {} => {:?}", file!(), line!(), why);
			return data.reply(Message(&why.tldr)).await;
		},
	};

	let rand = rand::thread_rng().gen_range(0..map_names.len());

	return data.reply(Message(&format!("🎲 {}", map_names[rand]))).await;
}
