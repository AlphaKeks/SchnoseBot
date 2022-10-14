use std::env;

use gokz_rs::functions::{get_place, get_recent};
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
	get_string, is_mention, is_steamid, retrieve_steam_id, timestring, Target, UserSchema,
};
use crate::SchnoseCommand;

pub fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	cmd.name("recent")
		.description("Check a player's most recent personal best")
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

	let target = if let Some(target) = get_string("target", opts) {
		if is_steamid(&target) {
			Target::SteamID(SteamId(target))
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
		Target::SteamID(steam_id) => PlayerIdentifier::SteamId(steam_id),
		Target::Name(name) => match SteamId::get(&PlayerIdentifier::Name(name), &client).await {
			Ok(steam_id) => PlayerIdentifier::SteamId(steam_id),
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
						Some(steam_id) => PlayerIdentifier::SteamId(steam_id),
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

	let recent = match get_recent(&player, &client).await {
		Ok(rec) => rec,
		Err(why) => {
			log::error!("`get_recent`: {:#?}", why);

			return SchnoseCommand::Message(why.tldr);
		}
	};

	let mode = Mode::from(recent.mode.clone());
	let (discord_timestamp, fancy) =
		match chrono::NaiveDateTime::parse_from_str(&recent.created_on, "%Y-%m-%dT%H:%M:%S") {
			Ok(parsed_time) => (
				format!("<t:{}:R>", parsed_time.timestamp()),
				format!("{} GMT", parsed_time.format("%d/%m/%Y - %H:%H:%S")),
			),
			Err(_) => (String::from(" "), String::from(" ")),
		};

	let place = match get_place(&recent, &client).await {
		Ok(place) => format!("[#{}]", place.0),
		Err(_) => String::from(" "),
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
		.url(format!(
			"https://kzgo.eu/maps/{}?{}=",
			&recent.map_name,
			&mode.fancy_short().to_lowercase()
		))
		.thumbnail(format!(
			"https://raw.githubusercontent.com/KZGlobalTeam/map-images/master/images/{}.jpg",
			&recent.map_name
		))
		.field(
			format!(
				"{} {}",
				mode.fancy_short(),
				if &recent.teleports > &0 { "TP" } else { "PRO" }
			),
			format!(
				"> {} {}\n> {}",
				timestring(recent.time),
				place,
				discord_timestamp
			),
			true,
		)
		.footer(|f| {
			let icon_url = env::var("ICON").unwrap_or(String::from("unknown"));

			f.text(fancy).icon_url(icon_url)
		})
		.to_owned();

	SchnoseCommand::Embed(embed)
}
