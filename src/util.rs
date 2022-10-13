#![allow(dead_code)]
use std::{env, fmt::Write};

use bson::doc;
use gokz_rs::prelude::{Mode, SteamId};
use mongodb::Collection;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serenity::{json::Value, model::prelude::interaction::application_command::CommandDataOption};

#[derive(Debug, Serialize, Deserialize)]
pub enum Target {
	SteamID(SteamId),
	Mention(String),
	Name(String),
}

pub fn timestring(secs_float: f32) -> String {
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
		if let Some(_) = r.find(input) {
			return true;
		}
	}

	false
}

pub async fn retrieve_steam_id(
	user_id: String,
	collection: Collection<UserSchema>,
) -> Result<Option<SteamId>, String> {
	match collection
		.find_one(doc! { "discordID": user_id }, None)
		.await
	{
		Err(_) => Err(String::from("Failed to access database.")),
		Ok(document) => match document {
			Some(entry) => match entry.steamID {
				Some(steam_id) => Ok(Some(SteamId(steam_id))),
				None => Ok(None),
			},
			None => Err(String::from("User not in database.")),
		},
	}
}

pub async fn retrieve_mode(
	query: bson::Document,
	collection: Collection<UserSchema>,
) -> Result<Option<Mode>, String> {
	match collection.find_one(query, None).await {
		Err(_) => Err(String::from("Failed to access database.")),
		Ok(document) => match document {
			Some(entry) => match entry.mode {
				Some(mode) => Ok(Some(Mode::from(mode))),
				None => Ok(None),
			},
			None => Err(String::from("User not in database.")),
		},
	}
}

pub fn get_string(input: &str, opts: &[CommandDataOption]) -> Option<String> {
	for opt in opts {
		if let Some(value) = &opt.value {
			if input == &opt.name {
				if let Value::String(string) = value {
					return Some(string.to_owned());
				}
			}
		}
	}

	None
}

pub fn get_integer(input: &str, opts: &[CommandDataOption]) -> Option<i64> {
	for opt in opts {
		if let Some(value) = &opt.value {
			if input == &opt.name {
				if let Value::Number(number) = value {
					if let Some(int) = number.as_i64() {
						return Some(int);
					}
				}
			}
		}
	}

	None
}

pub fn get_float(input: &str, opts: &[CommandDataOption]) -> Option<f64> {
	for opt in opts {
		if let Some(value) = &opt.value {
			if input == &opt.name {
				if let Value::Number(number) = value {
					if let Some(float) = number.as_f64() {
						return Some(float);
					}
				}
			}
		}
	}

	None
}

pub fn get_bool(input: &str, opts: &[CommandDataOption]) -> Option<bool> {
	for opt in opts {
		if let Some(value) = &opt.value {
			if input == &opt.name {
				if let Value::Bool(bool) = value {
					return Some(bool.to_owned());
				}
			}
		}
	}

	None
}

pub async fn get_steam_avatar(steamid64: &Option<String>, client: &reqwest::Client) -> String {
	#[derive(Debug, Serialize, Deserialize)]
	struct Player {
		pub steamid: String,
		pub communityvisibilitystate: i32,
		pub profilestate: i32,
		pub personaname: String,
		pub commentpermission: i32,
		pub profileurl: String,
		pub avatar: String,
		pub avatarmedium: String,
		pub avatarfull: String,
		pub avatarhash: String,
		pub lastlogoff: i32,
		pub personastate: i32,
		pub realname: Option<String>,
		pub primaryclanid: String,
		pub timecreated: i32,
		pub personastateflags: i32,
		pub loccountrycode: String,
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
		Err(_) => {
			return default_url;
		}
	};

	let url = format!(
		"https://api.steampowered.com/ISteamUser/GetPlayerSummaries/v0002/?key={}&steamids={}",
		api_key,
		match steamid64 {
			Some(id) => id,
			None => {
				return default_url;
			}
		}
	);

	match client.get(url).send().await {
		Ok(data) => match data.json::<Wrapper>().await {
			Ok(json) => {
				let player = &json.response.players[0];

				return player.avatarfull.to_owned();
			}
			Err(_) => {
				return default_url;
			}
		},
		Err(_) => {
			return default_url;
		}
	}
}
