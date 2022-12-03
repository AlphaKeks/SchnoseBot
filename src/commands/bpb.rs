use {
	crate::{
		events::slash_commands::{
			GlobalState,
			InteractionResponseData::{Message, Embed},
		},
		schnose::Target,
		util::{self, *},
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
		.name("bpb")
		.description("Check a player's personal best on a bonus.")
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
				.name("player")
				.description("Specify a player.")
				.required(false)
		})
		.create_option(|opt| {
			opt.kind(CommandOptionType::Integer)
				.name("course")
				.description("Specify a course.")
				.required(false)
		});
}

pub(crate) async fn execute(mut state: GlobalState<'_>) -> anyhow::Result<()> {
	state.defer().await?;

	let map_name = state.get::<String>("map_name").expect("This option is marked as `required`.");
	let course = state.get::<u8>("course").unwrap_or(1);
	dbg!(&course);
	let target = Target::from(state.get::<String>("player"));
	let player = match target.to_player(state.user, state.db).await {
		Ok(player) => player,
		Err(why) => {
			log::warn!("[{}]: {} => {:?}", file!(), line!(), why);
			return state.reply(Message(&why)).await;
		},
	};
	let mode = match state.get::<String>("mode") {
		Some(mode_name) => Mode::from_str(&mode_name).expect("This must be valid at this point."),
		None => match retrieve_mode(state.user, state.db).await {
			Ok(mode) => mode,
			Err(why) => {
				log::warn!("[{}]: {} => {:?}", file!(), line!(), why);
				return state.reply(Message(&why)).await;
			},
		},
	};

	let global_maps = match get_maps(&state.req_client).await {
		Ok(maps) => maps,
		Err(why) => {
			log::warn!("[{}]: {} => {:?}", file!(), line!(), why);
			return state.reply(Message(&why.tldr)).await;
		},
	};

	let map = match is_global(&MapIdentifier::Name(map_name), &global_maps).await {
		Ok(map) => map,
		Err(why) => {
			log::warn!("[{}]: {} => {:?}", file!(), line!(), why);
			return state.reply(Message(&why.tldr)).await;
		},
	};

	let map_identifier = MapIdentifier::Name(map.name.clone());

	let (tp, pro) = join_all([
		get_pb(&player, &map_identifier, &mode, true, course, &state.req_client),
		get_pb(&player, &map_identifier, &mode, false, course, &state.req_client),
	])
	.await
	.into_iter()
	.collect_tuple()
	.expect("This cannot fail, look 6 lines up.");

	if let (&Err(_), &Err(_)) = (&tp, &pro) {
		return state.reply(Message("No BPBs found 😔.")).await;
	}

	let player_name = get_player_name((&tp, &pro));
	let (place_tp, place_pro) = (
		util::get_place(&tp, &state.req_client).await,
		util::get_place(&pro, &state.req_client).await,
	);
	let links = (util::get_replay_link(&tp).await, util::get_replay_link(&pro).await);

	let mut embed = CreateEmbed::default()
		.colour(state.colour)
		.title(format!(
			"[BPB {}] {} on {} (T{})",
			&course, &player_name, &map.name, &map.difficulty
		))
		.url(format!(
			"https://kzgo.eu/maps/{}?bonus={}&{}=",
			&map.name,
			&course,
			&mode.to_fancy().to_lowercase()
		))
		.thumbnail(state.thumbnail(&map.name))
		.field(
			"TP",
			format!(
				"{} {}",
				match &tp {
					Ok(rec) => format_time(rec.time),
					_ => String::from("😔"),
				},
				place_tp
			),
			true,
		)
		.field(
			"PRO",
			format!(
				"{} {}",
				match &pro {
					Ok(rec) => format_time(rec.time),
					_ => String::from("😔"),
				},
				place_pro
			),
			true,
		)
		.footer(|f| f.text(format!("Mode: {}", mode.to_fancy())).icon_url(&state.icon))
		.to_owned();

	attach_replay_links(&mut embed, links);

	return state.reply(Embed(embed)).await;
}
