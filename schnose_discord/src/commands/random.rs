use {
	crate::{prelude::InteractionResult, events::interactions::InteractionState},
	gokz_rs::global_api::get_mapcycle,
	rand::Rng,
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

pub(crate) async fn execute(state: &InteractionState<'_>) -> InteractionResult {
	let tier = state.get::<u8>("tier");

	let map_names = get_mapcycle(tier, state.req_client).await?;

	let rand = rand::thread_rng().gen_range(0..map_names.len());

	return Ok(format!("🎲 {}", map_names[rand]).into());
}
