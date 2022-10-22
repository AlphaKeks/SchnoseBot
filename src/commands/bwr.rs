use std::env;
use std::str::FromStr;

use bson::doc;
use futures::future::join_all;
use gokz_rs::global_api::{get_maps, get_wr, is_global};
use gokz_rs::prelude::*;
use serenity::builder::CreateEmbed;
use serenity::model::user::User;
use serenity::{
	builder::CreateApplicationCommand,
	model::prelude::{
		command::CommandOptionType, interaction::application_command::CommandDataOption,
	},
};

use crate::util::{get_integer, get_string, retrieve_mode, timestring, UserSchema};
use crate::SchnoseCommand;

pub fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	cmd.name("bwr")
		.description("Check the wr on a bonus")
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

	let course = match get_integer("course", opts) {
		Some(course) => course as u8,
		None => 1,
	};

	let requests = join_all(
		vec![
			get_wr(&MapIdentifier::ID(map.id), &mode, true, course, &client),
			get_wr(&MapIdentifier::ID(map.id), &mode, false, course, &client),
		]
		.into_iter(),
	)
	.await;

	if let (&Err(_), &Err(_)) = (&requests[0], &requests[1]) {
		return SchnoseCommand::Message(String::from("No WR found."));
	}

	let embed = CreateEmbed::default()
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
					Ok(rec) => timestring(rec.time),
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
					Err(_) => String::from(" "),
				}
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
				match &requests[1] {
					Ok(rec) => format!(
						"({})",
						match &rec.player_name {
							Some(name) => name,
							None => "unknown",
						}
					),
					Err(_) => String::from(" "),
				}
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
