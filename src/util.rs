use std::fmt::Write;

use gokz_rs::global_api::GOKZModeName;
use regex::Regex;
use serde::{Deserialize, Serialize};

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
	pub mode: Option<GOKZModeName>,
}

pub fn is_steamid(steam_id: &str) -> bool {
	let regex = Regex::new(r"STEAM_[0-1]:[0-1]:[0-9]+");

	if let Ok(r) = regex {
		if let Some(_) = r.find(steam_id) {
			return true;
		} else {
			return false;
		}
	}

	false
}
