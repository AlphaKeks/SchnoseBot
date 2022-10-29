use std::str::FromStr;

use bson::doc;

use gokz_rs::{
	global_api::{get_place, get_player, get_recent},
	prelude::*,
};
use serenity::{
	builder::{CreateApplicationCommand, CreateEmbed},
	model::{prelude::command::CommandOptionType, user::User},
};

use crate::{
	event_handler::interaction_create::{CommandOptions, SchnoseResponseData},
	util::{
		format_time, get_id_from_mention, retrieve_steam_id, sanitize_target, Target, UserSchema,
	},
};

pub fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	cmd.name("recent")
		.description("Check a player's most recent personal best.")
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
	let target_input = match opts.get_string("target") {
		Some(target) => sanitize_target(target),
		None => Target::None,
	};

	let client = reqwest::Client::new();

	let steam_id = match target_input {
		Target::None => {
			match retrieve_steam_id(doc! { "discordID": user.id.to_string() }, collection).await {
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
			}
		},
		Target::Mention(mention) => match get_id_from_mention(mention) {
			Ok(id) => match retrieve_steam_id(doc! { "discordID": id.to_string() }, collection)
				.await
			{
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
				Ok(player) => SteamID(player.steam_id),
			}
		},
	};

	let recent = match get_recent(&PlayerIdentifier::SteamID(steam_id), &client).await {
		Err(why) => {
			log::error!("[{}]: {} => {}\n{:#?}", file!(), line!(), "Failed to get recent.", why);

			return SchnoseResponseData::Message(why.tldr);
		},
		Ok(recent) => recent,
	};

	let mode = Mode::from_str(&recent.mode);

	let place = match get_place(&recent.id, &client).await {
		Err(why) => {
			log::error!("[{}]: {} => {}\n{:#?}", file!(), line!(), "Failed to get recent.", why);

			return SchnoseResponseData::Message(why.tldr);
		},
		Ok(place) => format!("[#{}]", place.0),
	};

	let (discord_timestamp, fancy) =
		match chrono::NaiveDateTime::parse_from_str(&recent.created_on, "%Y-%m-%dT%H:%M:%S") {
			Ok(parsed_time) => (
				format!("<t:{}:R>", parsed_time.timestamp()),
				format!("{} GMT", parsed_time.format("%d/%m/%Y - %H:%H:%S")),
			),
			Err(_) => (String::from(" "), String::from(" ")),
		};

	let embed = CreateEmbed::default()
		.color((116, 128, 194))
		.title(format!(
			"[PB] {} on {}",
			match &recent.player_name {
				Some(name) => name,
				None => "unknown",
			},
			&recent.map_name
		))
		.url(
			format!("https://kzgo.eu/maps/{}", &recent.map_name)
				+ &(match &mode {
					Ok(mode) => format!("?{}=", mode.fancy_short().to_lowercase()),
					Err(_) => String::new(),
				}),
		)
		.thumbnail(format!(
			"https://raw.githubusercontent.com/KZGlobalTeam/map-images/master/images/{}.jpg",
			&recent.map_name
		))
		.field(
			format!(
				"{} {}",
				match &mode {
					Ok(mode) => mode.fancy_short(),
					Err(_) => String::new(),
				},
				if &recent.teleports > &0 { "TP" } else { "PRO" }
			),
			format!("> {} {}\n> {}", format_time(recent.time), place, discord_timestamp),
			true,
		)
		.footer(|f| f.text(fancy).icon_url(&root.icon))
		.to_owned();

	return SchnoseResponseData::Embed(embed);
}
