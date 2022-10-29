#![allow(dead_code)]
use std::{env, fmt::Write, str::FromStr};

use bson::doc;
use gokz_rs::prelude::*;
use mongodb::Collection;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Target {
	SteamID(SteamID),
	Mention(String),
	Name(String),
	None,
}

pub fn format_time(secs_float: f32) -> String {
	let seconds = secs_float as u32;
	let hours = ((seconds / 3600) % 24) as u8;
	let seconds = seconds % 3600;
	let minutes = (seconds / 60) as u8;
	let seconds = seconds % 60;
	let millis = ((secs_float - (secs_float as u32) as f32) * 1000.0) as u16;

	let mut s = String::new();

	let _ = write!(&mut s, "{:02}:{:02}.{:03}", minutes, seconds, millis);

	if hours > 0 {
		s = format!("{:02}:{}", hours, s);
	}

	s
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code, non_snake_case)]
pub struct UserSchema {
	pub name: String,
	pub discordID: String,
	pub steamID: Option<String>,
	pub mode: Option<String>,
}

pub fn is_steamid(steam_id: &str) -> bool {
	let regex = Regex::new(r"^STEAM_[0-1]:[0-1]:[0-9]+");

	if let Ok(r) = regex {
		return r.is_match(steam_id);
	}

	false
}

pub fn is_mention(input: &str) -> bool {
	let regex = Regex::new(r"<@[0-9]+>");

	if let Ok(r) = regex {
		return r.is_match(input);
	}

	false
}

pub fn get_id_from_mention(mention: String) -> Result<u64, String> {
	match mention.split_once(">") {
		Some((s, _)) => match s.split_once("@") {
			Some((_, id)) => match id.parse::<u64>() {
				Ok(id) => return Ok(id),
				Err(why) => return Err(format!("Failed to parse mention: {}", why)),
			},
			None => return Err(String::from("Failed to parse mention.")),
		},
		None => return Err(String::from("Failed to parse mention.")),
	}
}

pub fn sanitize_target(target: String) -> Target {
	if is_steamid(&target) {
		Target::SteamID(SteamID(target))
	} else if is_mention(&target) {
		Target::Mention(target)
	} else {
		Target::Name(target)
	}
}

pub async fn retrieve_steam_id(
	user_id: String,
	collection: &Collection<UserSchema>,
) -> Result<Option<SteamID>, String> {
	match collection.find_one(doc! { "discordID": user_id.clone() }, None).await {
		Err(why) => {
			log::error!(
				"[{}]: {} => {}\n{:#?}",
				file!(),
				line!(),
				"Failed to access database.",
				why
			);

			Err(String::from("Failed to access database."))
		},
		Ok(document) => match document {
			Some(entry) => match entry.steamID {
				Some(steam_id) => Ok(Some(SteamID(steam_id))),
				None => Ok(None),
			},
			None => {
				log::error!("[{}]: {} => {}", file!(), line!(), "User not found in database.",);

				Err(String::from("User not found in database."))
			},
		},
	}
}

pub async fn retrieve_mode(
	query: bson::Document,
	collection: &Collection<UserSchema>,
) -> Result<Option<Mode>, String> {
	match collection.find_one(query.clone(), None).await {
		Err(why) => {
			log::error!(
				"[{}]: {} => {}\n{:#?}",
				file!(),
				line!(),
				"Failed to access database.",
				why
			);

			Err(String::from("Failed to access database."))
		},
		Ok(document) => match document {
			Some(entry) => match entry.mode {
				Some(mode) => Ok(Some(match Mode::from_str(&mode) {
					Ok(mode) => mode,
					Err(why) => return Err(why.tldr),
				})),
				None => Ok(None),
			},
			None => {
				log::error!(
					"[{}]: {} => {}",
					file!(),
					line!(),
					query.to_string() + " wasn't found in database.",
				);

				Err(String::from("User not in database."))
			},
		},
	}
}

pub async fn get_steam_avatar(steamid64: &Option<String>, client: &reqwest::Client) -> String {
	#[derive(Debug, Serialize, Deserialize)]
	struct Player {
		pub steamid: Option<String>,
		pub communityvisibilitystate: Option<i32>,
		pub profilestate: Option<i32>,
		pub personaname: Option<String>,
		pub commentpermission: Option<i32>,
		pub profileurl: Option<String>,
		pub avatar: Option<String>,
		pub avatarmedium: Option<String>,
		pub avatarfull: Option<String>,
		pub avatarhash: Option<String>,
		pub lastlogoff: Option<i32>,
		pub personastate: Option<i32>,
		pub realname: Option<String>,
		pub primaryclanid: Option<String>,
		pub timecreated: Option<i32>,
		pub personastateflags: Option<i32>,
		pub loccountrycode: Option<String>,
	}

	#[derive(Debug, Serialize, Deserialize)]
	struct Response {
		pub players: Vec<Player>,
	}

	#[derive(Debug, Serialize, Deserialize)]
	struct Wrapper {
		pub response: Response,
	}

	let default_url = String::from("https://cdn.discordapp.com/attachments/981130651094900756/981130719537545286/churchOfSchnose.png");

	let api_key: String = match env::var("STEAM_API") {
		Ok(key) => key,
		Err(why) => {
			log::error!(
				"[{}]: {} => {}\n{:#?}",
				file!(),
				line!(),
				"Failed to get Steam Avatar.",
				why
			);

			return default_url;
		},
	};

	let url = format!(
		"https://api.steampowered.com/ISteamUser/GetPlayerSummaries/v0002/?key={}&steamids={}",
		api_key,
		match steamid64 {
			Some(id) => id,
			None => return default_url,
		}
	);

	match client.get(url).send().await {
		Ok(data) => match data.json::<Wrapper>().await {
			Ok(json) => {
				let player = &json.response.players[0];

				match &player.avatarfull {
					Some(avatar) => return avatar.to_owned(),
					None => return default_url,
				}
			},
			Err(why) => {
				log::error!(
					"[{}]: {} => {}\n{:#?}",
					file!(),
					line!(),
					"Failed to get Steam Avatar.",
					why
				);

				return default_url;
			},
		},
		Err(why) => {
			log::error!(
				"[{}]: {} => {}\n{:#?}",
				file!(),
				line!(),
				"Failed to get Steam Avatar.",
				why
			);

			return default_url;
		},
	}
}
