use {
	crate::{
		events::slash_command::{
			InteractionData,
			InteractionResponseData::{Message, Embed},
		},
		util::*,
	},
	anyhow::Result,
	bson::doc,
	itertools::Itertools,
	gokz_rs::{prelude::*, global_api::*},
	futures::future::join_all,
	serenity::{
		builder::{CreateApplicationCommand, CreateEmbed},
		model::prelude::command::CommandOptionType,
	},
};

pub fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	return cmd
		.name("pb")
		.description("Check a player's personal best on a map.")
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
		});
}

pub async fn execute(mut ctx: InteractionData<'_>) -> Result<()> {
	ctx.defer().await?;

	// sanitize user input
	let map = ctx.get_string("map_name").expect("This option is marked as `required`.");
	let mode = match ctx.get_string("mode") {
		Some(mode_name) => Mode::from_str(&mode_name)
			.expect("`mode_name` _has_ to be valid here. See the `register` function above."),
		None => {
			match ctx
				.db
				.find_one(doc! { "discordID": ctx.root.user.id.as_u64().to_string() }, None)
				.await
			{
				Ok(Some(entry)) => Mode::from_str(&entry.mode.unwrap_or(String::from("kz_timer")))
					.expect("Mode stored in the database _needs_ to be valid."),
				_ => Mode::KZTimer,
			}
		},
	};
	let player = match sanitize_target(ctx.get_string("player"), &ctx.db, &ctx.root).await {
		Some(target) => target,
		None => {
			return Ok(ctx
				.reply(Message("Please specify a player or save your own SteamID via `/setsteam`."))
				.await?)
		},
	};

	let client = reqwest::Client::new();

	let global_maps = match get_maps(&client).await {
		Ok(maps) => maps,
		Err(why) => {
			log::error!(
				"[{}]: {} => {}\n{:?}",
				file!(),
				line!(),
				"Failed to fetch global maps.",
				why
			);
			return Ok(ctx.reply(Message("Failed to fetch global maps.")).await?);
		},
	};

	let map = match is_global(&MapIdentifier::Name(map), &global_maps).await {
		Ok(map) => map,
		Err(why) => {
			log::warn!("[{}]: {} => {}\n{:?}", file!(), line!(), "Given map is not global.", why);
			return Ok(ctx.reply(Message("Please input a global map.")).await?);
		},
	};

	let steam_id = match player {
		Target::SteamID(steam_id) => steam_id,
		Target::Name(player_name) => {
			match get_player(&PlayerIdentifier::Name(player_name), &client).await {
				Ok(player) => SteamID(player.steam_id),
				Err(why) => {
					log::warn!(
						"[{}]: {} => {}\n{:?}",
						file!(),
						line!(),
						"Failed to get player from the GlobalAPI.",
						why
					);
					return Ok(ctx.reply(Message("Couldn't fetch player from GlobalAPI.")).await?);
				},
			}
		},
	};

	let player_identifier = PlayerIdentifier::SteamID(steam_id);
	let map_identifier = MapIdentifier::Name(map.name.clone());

	let (tp, pro) = join_all([
		get_pb(&player_identifier, &map_identifier, &mode, true, 0, &client),
		get_pb(&player_identifier, &map_identifier, &mode, false, 0, &client),
	])
	.await
	.into_iter()
	.collect_tuple()
	.unwrap();

	if let (&Err(_), &Err(_)) = (&tp, &pro) {
		return Ok(ctx.reply(Message("No PBs found 😔")).await?);
	}

	let player_name = match &tp {
		Ok(rec) => rec.player_name.clone().unwrap_or(String::from("unknown")),
		Err(_) => match &pro {
			Ok(rec) => rec.player_name.clone().unwrap_or(String::from("unknown")),
			Err(_) => {
				unreachable!("If both records had failed, we would have already returned earlier.")
			},
		},
	};

	let (place_tp, place_pro) = (
		match &tp {
			Ok(rec) => match get_place(&rec.id, &client).await {
				Ok(place) => format!("[#{}]", place.0),
				Err(_) => String::new(),
			},
			Err(_) => String::new(),
		},
		match &pro {
			Ok(rec) => match get_place(&rec.id, &client).await {
				Ok(place) => format!("[#{}]", place.0),
				Err(_) => String::new(),
			},
			Err(_) => String::new(),
		},
	);

	let mut embed = CreateEmbed::default()
		.color((116, 128, 194))
		.title(format!("[PB] {} on {} (T{})", &player_name, &map.name, &map.difficulty))
		.url(format!(
			"https://kzgo.eu/maps/{}?{}=",
			&map.name,
			&mode.to_fancy().to_lowercase()
		))
		.thumbnail(format!(
			"https://raw.githubusercontent.com/KZGlobalTeam/map-images/master/images/{}.jpg",
			&map.name
		))
		.field(
			"TP",
			format!(
				"{} {}",
				match &tp {
					Ok(rec) => format_time(rec.time),
					Err(_) => String::from("😔"),
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
					Err(_) => String::from("😔"),
				},
				place_pro
			),
			true,
		)
		.footer(|f| f.text(format!("Mode: {}", mode.to_fancy())).icon_url(&ctx.client.icon_url))
		.to_owned();

	let (tp_link, pro_link) = {
		let (mut tp_link, mut pro_link) = (String::new(), String::new());

		if let Ok(rec) = &tp {
			if rec.replay_id != 0 {
				if let Ok(link) = get_replay(rec.replay_id).await {
					tp_link = link;
				}
			}
		}

		if let Ok(rec) = &pro {
			if rec.replay_id != 0 {
				if let Ok(link) = get_replay(rec.replay_id).await {
					pro_link = link;
				}
			}
		}

		(tp_link, pro_link)
	};

	if tp_link.len() > 0 || pro_link.len() > 0 {
		let mut description = String::from("Download Replays:");

		if tp_link.len() > 0 {
			description.push_str(&format!(" [TP]({}) |", tp_link))
		}

		if pro_link.len() > 0 {
			description.push_str(&format!(" [PRO]({})", pro_link))
		}

		embed.description(description);
	}

	return Ok(ctx.reply(Embed(embed)).await?);
}
