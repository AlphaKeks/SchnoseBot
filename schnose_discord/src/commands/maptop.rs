use {
	crate::{
		prelude::InteractionResult,
		events::interactions::InteractionState,
		database::util as DB,
		formatting::{get_replay_links, format_time},
	},
	gokz_rs::{
		prelude::*,
		global_api::{get_maps, is_global, get_maptop},
	},
	serenity::{
		builder::{CreateApplicationCommand, CreateEmbed},
		model::prelude::command::CommandOptionType,
	},
};

pub(crate) fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	return cmd
		.name("maptop")
		.description("Check the Top 100 Records on a map.")
		.create_option(|opt| {
			opt.kind(CommandOptionType::String)
				.name("map_name")
				.description("Specify a map.")
				.required(true)
		})
		.create_option(|opt| {
			opt.kind(CommandOptionType::String)
				.name("mode")
				.description("Choose a mode.")
				.add_string_choice("KZT", "kz_timer")
				.add_string_choice("SKZ", "kz_simple")
				.add_string_choice("VNL", "kz_vanilla")
				.required(false)
		})
		.create_option(|opt| {
			opt.kind(CommandOptionType::String)
				.name("runtype")
				.description("TP/PRO")
				.add_string_choice("TP", "true")
				.add_string_choice("PRO", "false")
				.required(false)
		});
}

pub(crate) async fn execute(state: &mut InteractionState<'_>) -> InteractionResult {
	// Defer current interaction since this could take a while
	state.defer().await?;

	let map_name = state.get::<String>("map_name").expect("This option is marked as `required`.");

	let mode = match state.get::<String>("mode") {
		Some(mode_name) => mode_name
			.parse()
			.expect("The possible values for this are hard-coded and should never be invalid."),
		None => DB::fetch_mode(state.user, state.db, true).await?,
	};

	let runtype = match state.get::<String>("runtype") {
		Some(runtype) => runtype == "true",
		None => false,
	};

	let global_maps = get_maps(state.req_client).await?;

	let map = is_global(&MapIdentifier::Name(map_name), &global_maps).await?;

	let map_identifier = MapIdentifier::Name(map.name.clone());

	let leaderboard = get_maptop(&map_identifier, &mode, runtype, 0, state.req_client).await?;

	let link = get_replay_links(&Ok(leaderboard[0].clone())).await;

	let mut embeds: Vec<CreateEmbed> = Vec::new();
	let get_embed = |i| {
		let mut embed = CreateEmbed::default()
			.colour(state.colour)
			.title(format!(
				"[Top 100 {} {}] {} (T{})",
				&mode.to_fancy(),
				if runtype { "TP" } else { "PRO" },
				&map.name,
				&map.difficulty
			))
			.url(format!("{}?{}=", state.map_link(&map.name), &mode.to_fancy().to_lowercase()))
			.thumbnail(state.map_thumbnail(&map.name))
			.footer(|f| f.text(format!("Page: {}", i)).icon_url(state.icon))
			.to_owned();

		// only checking one of them is fine
		if !link.0.is_empty() {
			embed
				.description(format!("[Watch Replay]({}) | [Download Replay]({})", link.0, link.1));
		}
		embed
	};

	let mut temp = get_embed(1);

	let len = leaderboard.len();
	for (i, record) in leaderboard.into_iter().enumerate() {
		let first_page = i == 0;
		let full_page = i % 12 == 0;
		let last_page = i == len - 1;

		if !first_page && (full_page || last_page) {
			embeds.push(temp.clone());
			temp = get_embed(embeds.len() + 1);
		}

		temp.field(
			format!(
				"{} [#{}]",
				record.player_name.unwrap_or_else(|| String::from("unknown")),
				i + 1
			),
			format_time(record.time),
			true,
		);
	}

	Ok(embeds.into())
}
