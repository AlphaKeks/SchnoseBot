use std::str::FromStr;

use futures::future::join_all;
use gokz_rs::{
	global_api::{get_maps, get_pb, get_place, get_player, is_global},
	prelude::*,
};
use serenity::{
	builder::{CreateApplicationCommand, CreateEmbed},
	model::{prelude::command::CommandOptionType, user::User},
};

use bson::doc;

use crate::{
	event_handler::interaction_create::{CommandOptions, SchnoseResponseData},
	util::{
		format_time, get_id_from_mention, retrieve_mode, retrieve_steam_id, sanitize_target,
		Target, UserSchema,
	},
};

pub fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	cmd.name("bpb")
		.description("Check a player's personal best on a bonus")
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
			opt.kind(CommandOptionType::Integer)
				.name("course")
				.description("Specify a course")
				.required(false)
		})
		.create_option(|opt| {
			opt.kind(CommandOptionType::String)
				.name("target")
				.description("Specify a target.")
				.required(false)
		})
}

pub async fn run<'a>(
	opts: CommandOptions<'a>,
	collection: &mongodb::Collection<UserSchema>,
	user: &User,
	root: &crate::Schnose,
) -> SchnoseResponseData {
	// sanitize user input
	let map_input = match opts.get_string("map_name") {
		Some(map_name) => map_name,
		None => unreachable!("option is required"),
	};
	let mode_input = match opts.get_string("mode") {
		Some(mode_name) => match Mode::from_str(&mode_name) {
			Err(why) => {
				log::error!(
					"[{}]: {} => {}\n{:#?}",
					file!(),
					line!(),
					"Failed to parse mode.",
					why
				);

				return SchnoseResponseData::Message(why.tldr);
			},
			Ok(mode) => Some(mode),
		},
		None => match retrieve_mode(doc! { "discordID": user.id.to_string() }, collection).await {
			Err(why) => {
				log::error!("[{}]: {} => {}", file!(), line!(), why,);

				return SchnoseResponseData::Message(String::from(
					"You must either specify a mode or set a default one with `/mode`.",
				));
			},
			Ok(mode) => mode,
		},
	};
	let course = {
		let temp = opts.get_int("course").unwrap_or(1);
		if temp < 1 {
			1 as u8
		} else {
			temp as u8
		}
	};
	let target_input = match opts.get_string("target") {
		Some(target) => sanitize_target(target),
		None => Target::None,
	};

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
		Ok(maps) => maps,
	};

	let map = match is_global(&MapIdentifier::Name(map_input), &global_maps).await {
		Err(why) => {
			log::error!("[{}]: {} => {}\n{:#?}", file!(), line!(), "Failed to validate map.", why);

			return SchnoseResponseData::Message(why.tldr);
		},
		Ok(map) => map,
	};

	let mode = match mode_input {
		Some(mode) => mode,
		None => {
			log::error!("[{}]: {} => {}", file!(), line!(), "No mode specified.",);

			return SchnoseResponseData::Message(String::from(
				"You must either specify a mode or set a default one with `/mode`.",
			));
		},
	};

	let mut player_name = None;
	let steam_id = match target_input {
		Target::None => match retrieve_steam_id(user.id.to_string(), collection).await {
			Err(why) => {
				log::error!("[{}]: {} => {}", file!(), line!(), why,);

				return SchnoseResponseData::Message(String::from(
					"You must either specify a target or save your SteamID with `/setsteam`.",
				));
			},
			Ok(steam_id) => match steam_id {
				Some(steam_id) => steam_id,
				None => {
					log::error!("[{}]: {} => {}", file!(), line!(), "Failed to parse mode.",);
					return SchnoseResponseData::Message(String::from(
						"You must either specify a target or save your SteamID with `/setsteam`.",
					));
				},
			},
		},
		Target::Mention(mention) => match get_id_from_mention(mention) {
			Ok(id) => match retrieve_steam_id(id.to_string(), collection).await {
				Err(why) => {
					log::error!("[{}]: {} => {}", file!(), line!(), why,);

					return SchnoseResponseData::Message(String::from(
						"You must either specify a target or save your SteamID with `/setsteam`.",
					));
				},
				Ok(steam_id) => match steam_id {
					Some(steam_id) => steam_id,
					None => {
						log::error!("[{}]: {} => {}", file!(), line!(), "No SteamID specified.",);
						return SchnoseResponseData::Message(String::from(
						"You must either specify a target or save your SteamID with `/setsteam`.",
					));
					},
				},
			},
			Err(why) => {
				log::error!("[{}]: {} => {}", file!(), line!(), why);
				return SchnoseResponseData::Message(why);
			},
		},
		Target::SteamID(steam_id) => steam_id,
		Target::Name(input_name) => {
			match get_player(&PlayerIdentifier::Name(input_name), &client).await {
				Err(why) => {
					log::error!(
						"[{}]: {} => {}\n{:#?}",
						file!(),
						line!(),
						"Failed to get player from GlobalAPI",
						why
					);

					return SchnoseResponseData::Message(why.tldr);
				},
				Ok(player) => {
					player_name = Some(player.name);
					SteamID(player.steam_id)
				},
			}
		},
	};

	let player_identifier = PlayerIdentifier::SteamID(steam_id);
	let map_identifier = MapIdentifier::Name(map.name.clone());

	let requests = join_all([
		get_pb(&player_identifier, &map_identifier, &mode, true, course, &client),
		get_pb(&player_identifier, &map_identifier, &mode, false, course, &client),
	])
	.await;

	if let (&Err(_), &Err(_)) = (&requests[0], &requests[1]) {
		return SchnoseResponseData::Message(String::from("No BPB found."));
	}

	let player_name = match player_name {
		Some(name) => name,
		None => match &requests[0] {
			Ok(rec) => match &rec.player_name {
				Some(name) => name.to_owned(),
				None => String::from("unknown"),
			},
			Err(_) => match &requests[1] {
				Ok(rec) => match &rec.player_name {
					Some(name) => name.to_owned(),
					None => String::from("unknown"),
				},
				Err(_) => unreachable!("If both requests had failed we already returned early"),
			},
		},
	};

	let places = (
		match &requests[0] {
			Ok(rec) => match get_place(&rec.id, &client).await {
				Ok(place) => format!("[#{}]", place.0),
				Err(_) => String::new(),
			},
			Err(_) => String::new(),
		},
		match &requests[1] {
			Ok(rec) => match get_place(&rec.id, &client).await {
				Ok(place) => format!("[#{}]", place.0),
				Err(_) => String::new(),
			},
			Err(_) => String::new(),
		},
	);

	let embed = CreateEmbed::default()
		.color((116, 128, 194))
		.title(format!("[BPB {}] {} on {}", &course, &player_name, &map.name))
		.url(format!(
			"https://kzgo.eu/maps/{}?bonus={}&{}=",
			&map.name,
			course,
			&mode.fancy_short().to_lowercase()
		))
		.thumbnail(format!(
			"https://raw.githubusercontent.com/KZGlobalTeam/map-images/master/images/{}.jpg",
			&map.name
		))
		.field(
			"TP",
			format!(
				"{} {}",
				match &requests[0] {
					Ok(rec) => format_time(rec.time),
					Err(_) => String::from("😔"),
				},
				places.0
			),
			true,
		)
		.field(
			"PRO",
			format!(
				"{} {}",
				match &requests[1] {
					Ok(rec) => format_time(rec.time),
					Err(_) => String::from("😔"),
				},
				places.1
			),
			true,
		)
		.footer(|f| f.text(format!("Mode: {}", mode.fancy())).icon_url(&root.icon))
		.to_owned();

	return SchnoseResponseData::Embed(embed);
}
