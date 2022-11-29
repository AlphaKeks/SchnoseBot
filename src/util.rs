use {
	std::{env, fmt::Write},
	crate::db::UserSchema,
	bson::doc,
	gokz_rs::{prelude::*, global_api},
	mongodb::Collection,
	regex::Regex,
	serde::{Serialize, Deserialize},
	serenity::{builder::CreateEmbed, model::user::User},
};

pub(crate) struct Mention(pub u64);

impl Mention {
	pub fn from(input: &str) -> Option<Self> {
		let Ok(regex) = Regex::new(r#"<@[0-9]+>"#) else {
			return None;
		};

		if !regex.is_match(&input) {
			return None;
		}

		// input = "<@291585142164815873>"
		if let Some((left, _)) = input.split_once(">") {
			// left = "<@291585142164815873"
			if let Some((_, id)) = left.split_once("@") {
				// id = "291585142164815873"
				if let Ok(id) = id.parse::<u64>() {
					// id = 291585142164815873
					return Some(Mention(id));
				}
			}
		}

		return None;
	}
}

pub(crate) fn format_time(secs_float: f32) -> String {
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

pub(crate) async fn retrieve_mode(
	user: &User,
	collection: &Collection<UserSchema>,
) -> Result<Mode, String> {
	match collection.find_one(doc! { "discordID": user.id.to_string() }, None).await {
		Ok(document) => {
			if let Some(entry) = document {
				if let Some(mode) = entry.mode {
					// TODO: migrate to a proper database
					if mode.as_str() != "none" {
						let mode =
							Mode::from_str(&mode).expect("This must be valid at this point.");
						return Ok(mode);
					}
				}
			}
			return Err(String::from(
				"You need to specify a mode or set a default one via `/mode`.",
			));
		},
		Err(why) => {
			log::error!("[{}]: {} => {:?}", file!(), line!(), why);
			return Err(String::from("Failed to access database."));
		},
	}
}

type PB = Result<gokz_rs::global_api::records::top::Response, gokz_rs::prelude::Error>;
pub(crate) fn get_player_name(records: (&PB, &PB)) -> String {
	match records.0 {
		Ok(tp) => tp.player_name.clone().unwrap_or(String::from("unknown")),
		Err(_) => match records.1 {
			Ok(pro) => pro.player_name.clone().unwrap_or(String::from("unknown")),
			Err(_) => String::from("unknown"),
		},
	}
}

pub(crate) async fn get_place(record: &PB, client: &reqwest::Client) -> String {
	if let Ok(record) = record {
		if let Ok(place) = global_api::get_place(&record.id, client).await {
			return format!("[#{}]", place.0);
		}
	}
	return String::new();
}

pub(crate) async fn get_replay_link(record: &PB) -> String {
	if let Ok(record) = record {
		if record.replay_id != 0 {
			if let Ok(link) = global_api::get_replay(record.replay_id).await {
				return link;
			}
		}
	}

	return String::new();
}

pub(crate) fn attach_replay_links(
	embed: &mut CreateEmbed,
	links: (String, String),
) -> &mut CreateEmbed {
	let tp = links.0.len() > 0;
	let pro = links.1.len() > 0;

	if tp || pro {
		let mut description = String::from("Download Replays:");

		let text = if tp && !pro {
			format!(" [TP]({})", links.0)
		} else if !tp && pro {
			format!(" [PRO]({})", links.1)
		} else {
			format!(" [TP]({}) | [PRO]({})", links.0, links.1)
		};

		description.push_str(&text);
		embed.description(description);
		return embed;
	}

	return embed;
}

pub(crate) async fn get_steam_avatar(
	steam_id64: &Option<String>,
	client: &reqwest::Client,
) -> String {
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
		match steam_id64 {
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
