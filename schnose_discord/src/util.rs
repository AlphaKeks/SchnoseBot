use std::env;

/// Uses the Steam API to retreive a user's profile picture
pub(crate) async fn get_steam_avatar(
	steam_id64: &Option<String>,
	client: &gokz_rs::Client,
) -> String {
	#[derive(Debug, serde::Serialize, serde::Deserialize)]
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

	#[derive(Debug, serde::Serialize, serde::Deserialize)]
	struct Response {
		pub players: Vec<Player>,
	}

	#[derive(Debug, serde::Serialize, serde::Deserialize)]
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
				if let Some(url) = &player.avatarfull {
					return url.to_owned();
				}
				default_url
			},
			Err(why) => {
				log::error!(
					"[{}]: {} => {}\n{:#?}",
					file!(),
					line!(),
					"Failed to get Steam Avatar.",
					why
				);
				default_url
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
			default_url
		},
	}
}
