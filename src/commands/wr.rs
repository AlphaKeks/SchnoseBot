use gokz_rs::global_api::{GOKZMapIdentifier, GOKZModeIdentifier, GOKZModeName};
use gokz_rs::{get_maps, get_wr, validate_map};
use serenity::builder::CreateEmbed;
use serenity::json::Value;
use serenity::{
	builder::CreateApplicationCommand,
	model::prelude::{
		command::CommandOptionType, interaction::application_command::CommandDataOption,
	},
};

use crate::util::timestring;
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
				.required(true)
		})
}

pub async fn run(opts: &[CommandDataOption]) -> SchnoseCommand {
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
									println!("{:#?}", why);
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

	match (input_map, input_mode) {
		(Some(map), Some(mode)) => {
			name1 = map.name.clone();
			name2 = map.name.clone();

			mode1 = mode.clone();
			mode2 = mode.clone();
		}
		_ => unreachable!("Failed to access required command options"),
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
	let tp_time;
	let pro_time;

	match tp {
		Some(rec) => {
			map_name = Some(rec.map_name);
			tp_time = timestring(rec.time);
		}
		None => tp_time = String::from("😔"),
	};

	match pro {
		Some(rec) => {
			map_name = Some(rec.map_name);
			pro_time = timestring(rec.time);
		}
		None => pro_time = String::from("😔"),
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
		.field("TP", tp_time, true)
		.field("PRO", pro_time, true)
		.to_owned();

	SchnoseCommand::Embed(embed)
}
