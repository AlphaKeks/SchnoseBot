use std::env;

use bson::doc;
use gokz_rs::global_api::{GOKZMapIdentifier, GOKZModeIdentifier, GOKZModeName};
use gokz_rs::{get_maps, get_wr, validate_map};
use serenity::builder::CreateEmbed;
use serenity::json::Value;
use serenity::model::user::User;
use serenity::{
	builder::CreateApplicationCommand,
	model::prelude::{
		command::CommandOptionType, interaction::application_command::CommandDataOption,
	},
};

use crate::util::{timestring, UserSchema};
use crate::SchnoseCommand;

pub fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	cmd.name("wr")
		.description("Check the World Record of a map")
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
}

pub async fn run(
	user: &User,
	opts: &[CommandDataOption],
	mongo_client: &mongodb::Client,
) -> SchnoseCommand {
	let mut input_map = None;
	let mut input_mode = None;

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

			_ => (),
		}
	}

	let name1;
	let name2;
	let mode1;
	let mode2;

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
		match get_wr(
			GOKZMapIdentifier::Name(name1),
			0,
			GOKZModeIdentifier::Name(mode1),
			true,
			&client,
		)
		.await
		{
			Ok(rec) => Some(rec),
			Err(_) => None,
		},
		match get_wr(
			GOKZMapIdentifier::Name(name2),
			0,
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
		return SchnoseCommand::Message(String::from("No WRs found."));
	}

	let mut map_name = None;
	let mut tp_time = String::from("😔");
	let mut tp_player = String::from("unknown");
	let mut pro_time = String::from("😔");
	let mut pro_player = String::from("unknown");

	match tp {
		Some(rec) => {
			map_name = Some(rec.map_name);
			tp_time = timestring(rec.time);
			if let Some(name) = rec.player_name {
				tp_player = name
			}
		}
		None => (),
	};

	match pro {
		Some(rec) => {
			map_name = Some(rec.map_name);
			pro_time = timestring(rec.time);
			if let Some(name) = rec.player_name {
				pro_player = name
			}
		}
		None => (),
	};

	let embed = CreateEmbed::default()
		.color((116, 128, 194))
		.title(format!("[WR] {}", map_name.as_ref().unwrap()))
		.url(format!(
			"https://kzgo.eu/maps/{}",
			map_name.as_ref().unwrap()
		))
		.thumbnail(format!(
			"https://raw.githubusercontent.com/KZGlobalTeam/map-images/master/images/{}.jpg",
			&map_name.unwrap()
		))
		.field("TP", format!("{} ({})", tp_time, tp_player), true)
		.field("PRO", format!("{} ({})", pro_time, pro_player), true)
		.footer(|f| {
			let icon_url = env::var("ICON").unwrap_or(String::from("unknown"));

			f.text(format!(
				"Mode: {}",
				match input_mode {
					None => "unknown",
					Some(mode) => mode.fancy(),
				}
			))
			.icon_url(icon_url)
		})
		.to_owned();

	SchnoseCommand::Embed(embed)
}
