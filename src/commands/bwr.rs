use std::str::FromStr;

use futures::future::join_all;
use gokz_rs::{
	global_api::{get_maps, get_replay, get_wr, is_global},
	prelude::*,
};
use serenity::{
	builder::{CreateApplicationCommand, CreateEmbed},
	model::{prelude::command::CommandOptionType, user::User},
};

use bson::doc;

use crate::{
	event_handler::interaction_create::{CommandOptions, SchnoseResponseData},
	util::{format_time, retrieve_mode, UserSchema},
};

pub fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	cmd.name("bwr")
		.description("Check a the World Record on a bonus.")
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

	let map_identifier = MapIdentifier::Name(map.name.clone());

	let requests = join_all([
		get_wr(&map_identifier, &mode, true, course, &client),
		get_wr(&map_identifier, &mode, false, course, &client),
	])
	.await;

	if let (&Err(_), &Err(_)) = (&requests[0], &requests[1]) {
		return SchnoseResponseData::Message(String::from("No BWR found."));
	}

	let mut embed = CreateEmbed::default()
		.color((116, 128, 194))
		.title(format!("[BWR {}] {}", &course, &map.name))
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
				match &requests[0] {
					Ok(rec) => format!(
						"({})",
						match &rec.player_name {
							Some(name) => name,
							None => "unknown",
						}
					),
					Err(_) => String::new(),
				}
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
				match &requests[1] {
					Ok(rec) => format!(
						"({})",
						match &rec.player_name {
							Some(name) => name,
							None => "unknown",
						}
					),
					Err(_) => String::new(),
				}
			),
			true,
		)
		.footer(|f| f.text(format!("Mode: {}", mode.fancy())).icon_url(&root.icon))
		.to_owned();

	let link = {
		let mut temp = [String::new(), String::new()];

		if let Ok(record) = &requests[0] {
			if record.replay_id != 0 {
				match get_replay(record.replay_id).await {
					Ok(link) => temp[0] = link,
					Err(_) => (),
				}
			}
		}

		if let Ok(record) = &requests[1] {
			if record.replay_id != 0 {
				match get_replay(record.replay_id).await {
					Ok(link) => temp[1] = link,
					Err(_) => (),
				}
			}
		}

		temp
	};

	if link[0].len() > 0 || link[1].len() > 0 {
		let mut description = String::from("Download Replays:");

		if link[0].len() > 0 {
			description.push_str(&format!(" [TP]({}) |", link[0]))
		}

		if link[1].len() > 0 {
			description.push_str(&format!(" [PRO]({})", link[1]))
		}

		embed.description(description);
	}

	return SchnoseResponseData::Embed(embed);
}
