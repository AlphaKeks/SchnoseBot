use std::env;

use bson::doc;
use gokz_rs::global_api::{
	GOKZMapIdentifier, GOKZModeIdentifier, GOKZModeName, GOKZPlayerIdentifier,
};
use gokz_rs::{get_maps, get_pb, validate_map};
use serenity::builder::CreateEmbed;
use serenity::json::Value;
use serenity::model::user::User;
use serenity::{
	builder::CreateApplicationCommand,
	model::prelude::{
		command::CommandOptionType, interaction::application_command::CommandDataOption,
	},
};

use crate::util::{is_mention, is_steamid, retrieve_steam_id, timestring, Target, UserSchema};
use crate::SchnoseCommand;

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
				.description("Specify a mode.")
				.add_string_choice("KZT", "kz_timer")
				.add_string_choice("SKZ", "kz_simple")
				.add_string_choice("VNL", "kz_vanilla")
				.required(false)
		})
		.create_option(|opt| {
			opt.kind(CommandOptionType::Integer)
				.name("course")
				.description("Specify a course.")
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
	let mut input_map = None;
	let mut input_mode = None;
	let mut input_target = None;
	let mut course = 1;

	let client = reqwest::Client::new();

	for opt in opts {
		match opt.name.as_str() {
			"map_name" => match &opt.value {
				Some(val) => match val.to_owned() {
					Value::String(str) => {
						let global_maps = match get_maps(&client).await {
							Ok(maps) => maps,
							Err(why) => return SchnoseCommand::Message(why.tldr),
						};

						input_map =
							match validate_map(GOKZMapIdentifier::Name(str), global_maps).await {
								Ok(map) => Some(map),
								Err(why) => {
									return SchnoseCommand::Message(why.tldr);
								}
							}
					}
					_ => {
						return SchnoseCommand::Message(String::from(
							"Failed to deserialize input map.",
						))
					}
				},
				None => unreachable!("Failed to access required command option"),
			},

			"mode" => match &opt.value {
				Some(val) => match val.to_owned() {
					Value::String(mode_val) => {
						input_mode = match mode_val.as_str() {
							"kz_timer" => Some(GOKZModeName::kz_timer),
							"kz_simple" => Some(GOKZModeName::kz_simple),
							"kz_vanilla" => Some(GOKZModeName::kz_vanilla),
							_ => unreachable!("Invalid input mode"),
						}
					}
					_ => {
						return SchnoseCommand::Message(String::from(
							"Failed to deserialize input mode.",
						))
					}
				},
				None => unreachable!("Failed to access required command option"),
			},

			"course" => match &opt.value {
				Some(val) => match val.to_owned() {
					Value::Number(num) => match num.as_u64() {
						Some(num) => course = num as u8,
						None => (),
					},
					_ => (),
				},
				None => (),
			},

			"target" => match &opt.value {
				Some(val) => {
					if is_steamid(&val.to_string()) {
						input_target = Some(Target::SteamID(val.to_string()));
					} else if is_mention(&val.to_string()) {
						let collection = mongo_client
							.database("gokz")
							.collection::<UserSchema>("users");

						let id;
						if let Some(s) = val.to_string().split_once(">") {
							id = s.0.to_string();
						} else {
							id = String::new();
						}

						match retrieve_steam_id(id, collection).await {
							Ok(steam_id) => match steam_id {
								Some(steam_id) => input_target = Some(Target::SteamID(steam_id)),
								None => return SchnoseCommand::Message(String::from("You need to provide a target (steamID, name or mention) or set a default steamID with `/setsteam`."))
							},
							Err(why) => return SchnoseCommand::Message(why),
						}
					} else {
						input_target = Some(Target::Name(val.to_string()));
					}
				}
				None => (),
			},

			_ => (),
		}
	}

	if let None = input_target {
		let collection = mongo_client
			.database("gokz")
			.collection::<UserSchema>("users");

		match retrieve_steam_id(user.id.to_string(), collection).await {
			Ok(steam_id) => match steam_id {
				Some(steam_id) => input_target = Some(Target::SteamID(steam_id)),
				None => return SchnoseCommand::Message(String::from(
					"You need to provide a target (steamID, name or mention) or set a default steamID with `/setsteam`.",
				)),
			},
			Err(why) => return SchnoseCommand::Message(why),
		}
	}

	let name1;
	let name2;
	let mode1;
	let mode2;

	let player1 = match input_target {
		Some(Target::SteamID(steam_id)) => GOKZPlayerIdentifier::SteamID(steam_id),
		Some(Target::Name(name)) => GOKZPlayerIdentifier::Name(name),
		_ => return SchnoseCommand::Message(String::from("You need to provide a target (steamID, name or mention) or set a default steamID with `/setsteam`."))
	};
	let player2 = player1.clone();

	match input_map {
		Some(map) => {
			name1 = map.name.clone();
			name2 = map.name;
		}
		_ => unreachable!("Failed to access required command options"),
	}

	match input_mode.clone() {
		Some(mode) => {
			mode1 = mode.clone();
			mode2 = mode;
		}
		None => {
			let database = mongo_client
				.database("gokz")
				.collection::<UserSchema>("users");

			match database
				.find_one(doc! { "discordID": user.id.to_string() }, None)
				.await
			{
				Err(_) => {
					return SchnoseCommand::Message(String::from("Failed to access database."))
				}
				Ok(document) => match document {
					None => {
						return SchnoseCommand::Message(String::from(
							"You need to specify a mode or set a default one with `/mode`.",
						));
					}
					Some(doc) => match doc.mode {
						Some(mode) => {
							mode1 = mode.clone();
							mode2 = mode.clone();
							input_mode = Some(mode);
						}
						None => {
							return SchnoseCommand::Message(String::from(
								"You need to specify a mode or set a default one with `/mode`.",
							));
						}
					},
				},
			}
		}
	}

	let (tp, pro) = (
		match get_pb(
			player1,
			GOKZMapIdentifier::Name(name1),
			course,
			GOKZModeIdentifier::Name(mode1),
			true,
			&client,
		)
		.await
		{
			Ok(rec) => Some(rec),
			Err(_) => None,
		},
		match get_pb(
			player2,
			GOKZMapIdentifier::Name(name2),
			course,
			GOKZModeIdentifier::Name(mode2),
			false,
			&client,
		)
		.await
		{
			Ok(rec) => Some(rec),
			Err(_) => None,
		},
	);

	if let (&None, &None) = (&tp, &pro) {
		return SchnoseCommand::Message(String::from("No PB found."));
	}

	let mut map_name = None;
	let mut tp_time = String::from("😔");
	let mut pro_time = String::from("😔");
	let mut player_name = String::from("unknown");

	match tp {
		Some(rec) => {
			map_name = Some(rec.map_name);
			tp_time = timestring(rec.time);
			if let Some(name) = rec.player_name {
				player_name = name;
			}
		}
		None => (),
	};

	match pro {
		Some(rec) => {
			map_name = Some(rec.map_name);
			pro_time = timestring(rec.time);
			if let Some(name) = rec.player_name {
				player_name = name;
			}
		}
		None => (),
	};

	let embed = CreateEmbed::default()
		.color((116, 128, 194))
		.title(format!(
			"[BPB {}] {} on {}",
			course,
			player_name.clone(),
			match &map_name {
				Some(s) => s,
				None => "unknown map",
			}
		))
		.url(format!(
			"https://kzgo.eu/maps/{}?bonus={}&{}=",
			if let Some(s) = &map_name { s } else { "" },
			course,
			match input_mode.clone() {
				Some(mode) => mode.fancy_short().to_lowercase(),
				None => String::from("kzt"),
			}
		))
		.thumbnail(match &map_name {
			Some(s) => format!(
				"https://raw.githubusercontent.com/KZGlobalTeam/map-images/master/images/{}.jpg",
				s
			),
			None => String::from("https://kzgo.eu/kz_default.png"),
		})
		.field("TP", format!("{}", tp_time), true)
		.field("PRO", format!("{}", pro_time), true)
		.footer(|f| {
			let icon_url = env::var("ICON").unwrap_or(String::from("unknown"));

			f.text(format!(
				"Mode: {} | Player: {}",
				match input_mode {
					None => "unknown",
					Some(mode) => mode.fancy(),
				},
				player_name
			))
			.icon_url(icon_url)
		})
		.to_owned();

	SchnoseCommand::Embed(embed)
}
