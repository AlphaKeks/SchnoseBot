use std::env;
use std::str::FromStr;

use bson::doc;
use futures::future::join_all;
use gokz_rs::global_api::{get_maps, get_pb, get_place, get_player, is_global};
use gokz_rs::prelude::*;
use serenity::builder::CreateEmbed;
use serenity::model::user::User;
use serenity::{
	builder::CreateApplicationCommand,
	model::prelude::{
		command::CommandOptionType, interaction::application_command::CommandDataOption,
	},
};

use crate::util::{
	get_string, is_mention, is_steamid, retrieve_mode, retrieve_steam_id, timestring, Target,
	UserSchema,
};
use crate::SchnoseCommand;

pub fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	cmd.name("pb")
		.description("Check a player's personal best on a map")
		.create_option(|opt| {
			opt.kind(CommandOptionType::String)
				.name("map_name")
				.description("Specify a map.")
				.required(true)
		})
		.create_option(|opt| {
			opt.kind(CommandOptionType::String)
				.name("mode")
				.description("Specify a mode.")
				.add_string_choice("KZT", "kz_timer")
				.add_string_choice("SKZ", "kz_simple")
				.add_string_choice("VNL", "kz_vanilla")
				.required(false)
		})
		.create_option(|opt| {
			opt.kind(CommandOptionType::String)
				.name("target")
				.description("Specify a target.")
				.required(false)
		})
}

pub async fn run(
	user: &User,
	opts: &[CommandDataOption],
	mongo_client: &mongodb::Client,
) -> SchnoseCommand {
	let client = reqwest::Client::new();

	let map = match get_string("map_name", opts) {
		Some(map_name) => {
			let global_maps = match get_maps(&client).await {
				Ok(maps) => maps,
				Err(why) => {
					log::error!("`get_maps`: {:#?}", why);

					return SchnoseCommand::Message(why.tldr);
				}
			};

			match is_global(&MapIdentifier::Name(map_name), &global_maps).await {
				Ok(map) => map,
				Err(why) => {
					log::error!("`is_global`: {:#?}", why);

					return SchnoseCommand::Message(why.tldr);
				}
			}
		}
		None => unreachable!("Failed to access required command option"),
	};

	let mode = if let Some(mode_name) = get_string("mode", opts) {
		match Mode::from_str(&mode_name) {
			Err(why) => return SchnoseCommand::Message(why.tldr),
			Ok(mode) => mode,
		}
	} else {
		let collection = mongo_client
			.database("gokz")
			.collection::<UserSchema>("users");

		match retrieve_mode(doc! { "discordID": user.id.to_string() }, collection).await {
			Ok(mode) => match mode {
				Some(mode) => mode,
				None => {
					return SchnoseCommand::Message(String::from(
						"You need to specify a mode or set a default steamID with `/mode`.",
					))
				}
			},
			Err(why) => {
				log::error!("`retrieve_mode`: {:#?}", why);

				return SchnoseCommand::Message(why);
			}
		}
	};

	let target = if let Some(target) = get_string("target", opts) {
		if is_steamid(&target) {
			Target::SteamID(SteamID(target))
		} else if is_mention(&target) {
			let collection = mongo_client
				.database("gokz")
				.collection::<UserSchema>("users");

			let id;
			if let Some(s) = target.split_once(">") {
				if let Some(s) = s.0.split_once("@") {
					id = s.1.to_owned();
				} else {
					id = String::new();
				}
			} else {
				id = String::new();
			}

			match retrieve_steam_id(id, collection).await {
				Ok(steam_id) => match steam_id {
					Some(steam_id) => Target::SteamID(steam_id),
					None => return SchnoseCommand::Message(String::from("You need to provide a target (steamID, name or mention) or set a default steamID with `/setsteam`."))
				}
				Err(why) => {
					log::error!("`retrieve_steam_id`: {:#?}", why);

					return SchnoseCommand::Message(why)
				}
			}
		} else {
			Target::Name(target)
		}
	} else {
		let collection = mongo_client
			.database("gokz")
			.collection::<UserSchema>("users");

		match retrieve_steam_id(user.id.to_string(), collection).await {
				Ok(steam_id) => match steam_id {
					Some(steam_id) => Target::SteamID(steam_id),
					None => return SchnoseCommand::Message(String::from("You need to provide a target (steamID, name or mention) or set a default steamID with `/setsteam`."))
				},
				Err(why) => {
					log::error!("`retrieve_steam_id`: {:#?}", why);

					return SchnoseCommand::Message(why)
				}
			}
	};

	let player = match target {
		Target::SteamID(steam_id) => PlayerIdentifier::SteamID(steam_id),
		Target::Name(name) => match get_player(&PlayerIdentifier::Name(name), &client).await {
			Ok(player) => PlayerIdentifier::SteamID(SteamID(player.steam_id)),
			Err(why) => {
				log::error!("`SteamId::get()`: {:#?}", why);

				return SchnoseCommand::Message(why.tldr.to_owned());
			}
		},
		Target::Mention(mention) => {
			let collection = mongo_client
				.database("gokz")
				.collection::<UserSchema>("users");

			match retrieve_steam_id(mention, collection).await {
					Ok(steam_id) => match steam_id {
						Some(steam_id) => PlayerIdentifier::SteamID(steam_id),
						None => return SchnoseCommand::Message(String::from(
							"You need to provide a target (steamID, name or mention) or set a default steamID with `/setsteam`.",
						)),
					},
					Err(why) => {
						log::error!("`retrieve_steam_id`: {:#?}", why);

						return SchnoseCommand::Message(why)
					},
				}
		}
	};

	let requests = join_all(
		vec![
			get_pb(
				&player,
				&MapIdentifier::Name(map.name.clone()),
				&mode,
				true,
				0,
				&client,
			),
			get_pb(
				&player,
				&MapIdentifier::Name(map.name.clone()),
				&mode,
				false,
				0,
				&client,
			),
		]
		.into_iter(),
	)
	.await;

	if let (&Err(_), &Err(_)) = (&requests[0], &requests[1]) {
		return SchnoseCommand::Message(String::from("No PB found."));
	}

	let player = match &requests[0] {
		Ok(rec) => match &rec.player_name {
			Some(name) => name.to_owned(),
			None => String::from("unknown"),
		},
		Err(_) => match &requests[1] {
			Ok(rec) => match &rec.player_name {
				Some(name) => name.to_owned(),
				None => String::from("unknown"),
			},
			Err(_) => String::from("unknown"),
		},
	};

	let places = (
		match &requests[0] {
			Ok(rec) => match get_place(&rec.id, &client).await {
				Ok(place) => format!("[#{}]", place.0),
				Err(_) => String::from(" "),
			},
			Err(_) => String::from(" "),
		},
		match &requests[1] {
			Ok(rec) => match get_place(&rec.id, &client).await {
				Ok(place) => format!("[#{}]", place.0),
				Err(_) => String::from(" "),
			},
			Err(_) => String::from(" "),
		},
	);

	let embed = CreateEmbed::default()
		.color((116, 128, 194))
		.title(format!("[PB] {} on {}", &player, &map.name))
		.url(format!(
			"https://kzgo.eu/maps/{}?{}=",
			&map.name,
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
					Ok(rec) => timestring(rec.time),
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
					Ok(rec) => timestring(rec.time),
					Err(_) => String::from("😔"),
				},
				places.1
			),
			true,
		)
		.footer(|f| {
			let icon_url = env::var("ICON").unwrap_or(String::from("unknown"));

			f.text(format!("Mode: {}", mode.fancy())).icon_url(icon_url)
		})
		.to_owned();

	SchnoseCommand::Embed(embed)
}
