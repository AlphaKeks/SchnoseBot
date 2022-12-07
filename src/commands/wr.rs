use {
	crate::{
		events::slash_commands::{InteractionState, InteractionResponseData::*},
		schnose::InteractionResult,
		db::retrieve_mode,
		util::*,
		gokz,
	},
	futures::future::join_all,
	gokz_rs::{prelude::*, global_api::*},
	itertools::Itertools,
	serenity::{
		builder::{CreateApplicationCommand, CreateEmbed},
		model::prelude::command::CommandOptionType,
	},
};

pub(crate) fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	return cmd
		.name("wr")
		.description("Check the World Record on a map.")
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
		});
}

pub(crate) async fn execute(state: &mut InteractionState<'_>) -> InteractionResult {
	// Defer current interaction since this could take a while
	state.defer().await?;

	let map_name = state.get::<String>("map_name").expect("This option is marked as `required`.");

	let mode = match state.get::<String>("mode") {
		Some(mode_name) => Mode::from_str(&mode_name)
			.expect("The possible values for this are hard-coded and should never be invalid."),
		None => retrieve_mode(state.user, state.db).await?,
	};

	let global_maps = get_maps(&state.req_client).await?;

	let map = is_global(&MapIdentifier::Name(map_name), &global_maps).await?;

	let map_identifier = MapIdentifier::Name(map.name.clone());

	let (tp, pro) = join_all([
		get_wr(&map_identifier, &mode, true, 0, &state.req_client),
		get_wr(&map_identifier, &mode, false, 0, &state.req_client),
	])
	.await
	.into_iter()
	.collect_tuple()
	.expect("This cannot fail, look 6 lines up.");

	if let (&Err(_), &Err(_)) = (&tp, &pro) {
		return Ok(Message("No WRs found 😔.".into()));
	}

	let links = (gokz::get_replay_link(&tp).await, gokz::get_replay_link(&pro).await);

	let mut embed = CreateEmbed::default()
		.colour(state.colour)
		.title(format!("[WR] {} (T{})", &map.name, &map.difficulty))
		.url(format!(
			"https://kzgo.eu/maps/{}?{}=",
			&map.name,
			&mode.to_fancy().to_lowercase()
		))
		.thumbnail(state.thumbnail(&map.name))
		.field(
			"TP",
			format!(
				"{} ({})",
				match &tp {
					Ok(rec) => format_time(rec.time),
					_ => String::from("😔"),
				},
				{
					let mut name = "unknown";
					if let Ok(rec) = &tp {
						if let Some(player_name) = &rec.player_name {
							name = &player_name;
						}
					}
					name
				}
			),
			true,
		)
		.field(
			"PRO",
			format!(
				"{} ({})",
				match &pro {
					Ok(rec) => format_time(rec.time),
					_ => String::from("😔"),
				},
				{
					let mut name = "unknown";
					if let Ok(rec) = &pro {
						if let Some(player_name) = &rec.player_name {
							name = &player_name;
						}
					}
					name
				}
			),
			true,
		)
		.footer(|f| f.text(format!("Mode: {}", mode.to_fancy())).icon_url(&state.icon))
		.to_owned();

	attach_replay_links(&mut embed, links);

	return Ok(Embed(embed));
}
