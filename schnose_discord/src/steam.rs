use log::error;

/// Uses the Steam API to retreive a user's profile picture
pub async fn get_steam_avatar(
	steam_id64: &str, default_url: &str, steam_api_key: &str, client: &gokz_rs::Client,
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

	let url = format!(
		"https://api.steampowered.com/ISteamUser/GetPlayerSummaries/v0002/?key={steam_api_key}&steamids={steam_id64}"
	);

	match client.get(url).send().await {
		Ok(data) => match data.json::<Wrapper>().await {
			Ok(json) => {
				let player = &json.response.players[0];
				if let Some(url) = &player.avatarfull {
					return url.clone();
				}
				String::from(default_url)
			}
			Err(why) => {
				error!("Failed to get Steam Avatar: {:?}", why);
				String::from(default_url)
			}
		},
		Err(why) => {
			error!("Failed to get Steam Avatar: {:?}", why);
			String::from(default_url)
		}
	}
}
