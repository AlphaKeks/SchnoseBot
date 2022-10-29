use std::str::FromStr;

use futures::future::join_all;
use gokz_rs::{
	global_api::{get_maps, get_wr, is_global},
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
	cmd.name("wr")
		.description("Check a the World Record on a map.")
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
		get_wr(&map_identifier, &mode, true, 0, &client),
		get_wr(&map_identifier, &mode, false, 0, &client),
	])
	.await;

	if let (&Err(_), &Err(_)) = (&requests[0], &requests[1]) {
		return SchnoseResponseData::Message(String::from("No WR found."));
	}

	let embed = CreateEmbed::default()
		.color((116, 128, 194))
		.title(format!("[WR] {}", &map.name))
		.url(format!(
			"https://kzgo.eu/maps/{}?&{}=",
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

	return SchnoseResponseData::Embed(embed);
}
