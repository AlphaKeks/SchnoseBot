use {
	std::{env, fmt::Write},
	crate::db::UserSchema,
	bson::doc,
	gokz_rs::prelude::*,
	regex::Regex,
	serde::{Serialize, Deserialize},
	serenity::model::user::User,
};

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

	return s;
}

pub fn is_steamid(steam_id: &str) -> bool {
	let regex = Regex::new(r"^STEAM_[0-1]:[0-1]:[0-9]+");

	if let Ok(r) = regex {
		return r.is_match(steam_id);
	}

	return false;
}

pub fn is_mention(input: &str) -> bool {
	let regex = Regex::new(r"<@[0-9]+>");

	if let Ok(r) = regex {
		return r.is_match(input);
	}

	return false;
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

#[derive(Debug, Clone)]
pub enum Target {
	SteamID(SteamID),
	Name(String),
}

pub async fn sanitize_target(
	target: Option<String>,
	db_collection: &mongodb::Collection<UserSchema>,
	user: &User,
) -> Option<Target> {
	let Some(target) = target else {
		let Ok(Some(entry)) = db_collection.find_one(doc! { "discordID": user.id.to_string() }, None).await else {
			return None;
		};

		if let Some(steam_id) = entry.steamID {
			return Some(Target::SteamID(SteamID(steam_id)));
		}

		return None;
	};

	if is_steamid(&target) {
		return Some(Target::SteamID(SteamID(target)));
	} else if is_mention(&target) {
		let Ok(discord_id) = get_id_from_mention(target) else {
			return None;
		};

		let Ok(Some(entry)) = db_collection.find_one(doc! { "discordID": discord_id.to_string() }, None).await else {
			return None;
		};

		if let Some(steam_id) = entry.steamID {
			return Some(Target::SteamID(SteamID(steam_id)));
		}

		return None;
	} else {
		return Some(Target::Name(target));
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

	let default_url = env::var("ICON_URL")
		.expect("The bot should've crashed before this point if `ICON_URL` didn't exist.");

	let api_key: String = match env::var("STEAM_API") {
		Ok(key) => key,
		Err(why) => {
			log::error!("[{}]: {} => {}\n{:#?}", file!(), line!(), "No Steam API Key found.", why);

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
