use std::env;

use chrono::TimeZone;
use gokz_rs::get_recent;
use gokz_rs::global_api::GOKZPlayerIdentifier;
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
	let mut input_target = None;

	let client = reqwest::Client::new();

	for opt in opts {
		match opt.name.as_str() {
			"target" => match &opt.value {
				Some(val) => match val.to_owned() {
					Value::String(str) => {
						if is_steamid(&str) {
							if let None = input_target {
								input_target = Some(Target::SteamID(str));
							}
						} else if is_mention(&str) {
							let collection = mongo_client
								.database("gokz")
								.collection::<UserSchema>("users");

							let id;
							if let Some(s) = str.split_once(">") {
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
							input_target = Some(Target::Name(str));
						}
					}
					_ => {
						return SchnoseCommand::Message(String::from(
							"Please input a valid target.",
						))
					}
				},
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

	let player = match input_target {
		Some(Target::SteamID(steam_id)) => GOKZPlayerIdentifier::SteamID(steam_id),
		Some(Target::Name(name)) => GOKZPlayerIdentifier::Name(name),
		_ => return SchnoseCommand::Message(String::from("You need to provide a target (steamID, name or mention) or set a default steamID with `/setsteam`."))
	};

	let recent = match get_recent(player, &client).await {
		Ok(rec) => Some(rec),
		Err(_) => return SchnoseCommand::Message(String::from("No recent PB found.")),
	};

	let mut map_name = None;
	let mut time = String::from("😔");
	let mut unixtime: u64 = 0;
	let mut fancy_time = String::from(" ");
	let mut player_name = String::from("unknown");
	let mut mode = None;
	let mut runtype = String::from(" ");

	match recent {
		Some(rec) => {
			map_name = Some(rec.map_name);
			time = timestring(rec.time);

			if let Ok(parsed_time) =
				chrono::NaiveDateTime::parse_from_str(&rec.created_on, "%Y-%m-%dT%H:%M:%S")
			{
				unixtime = parsed_time.timestamp() as u64;
				fancy_time = format!("{} GMT", parsed_time.format("%d/%m/%Y - %H:%M:%S"))
			}

			if let Some(name) = rec.player_name {
				player_name = name;
			}

			mode = Some(rec.mode);
			if rec.teleports > 0 {
				runtype = String::from("TP");
			} else {
				runtype = String::from("PRO");
			}
		}
		None => (),
	};

	let embed = CreateEmbed::default()
		.color((116, 128, 194))
		.title(format!(
			"[PB] {} on {}",
			player_name.clone(),
			match &map_name {
				Some(s) => s,
				None => "unknown map",
			}
		))
		.url(format!(
			"https://kzgo.eu/maps/{}?{}=",
			if let Some(s) = &map_name { s } else { "" },
			match mode.clone() {
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
		.field(
			format!(
				"{} {}",
				match mode.clone() {
					Some(mode) => mode.fancy_short(),
					None => "unknown",
				},
				runtype
			),
			format!("> {} <t:{}:R>", time, unixtime),
			true,
		)
		.footer(|f| {
			let icon_url = env::var("ICON").unwrap_or(String::from("unknown"));

			f.text(fancy_time).icon_url(icon_url)
		})
		.to_owned();

	SchnoseCommand::Embed(embed)
}
