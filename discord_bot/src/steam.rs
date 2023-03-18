//! Steam WebAPI functions.
#![allow(unused)]

use {
	crate::Error,
	color_eyre::{eyre::eyre, Result as Eyre},
	serde::{Deserialize, Serialize},
};

pub async fn get_steam_avatar(
	steam_api_key: &str,
	steam_id64: u64,
	client: &gokz_rs::Client,
) -> Eyre<String> {
	let response = client
		.get(format!("https://api.steampowered.com/ISteamUser/GetPlayerSummaries/v0002/?key={steam_api_key}&steamids={steam_id64}"))
		.send()
		.await?;

	let mut json = response
		.json::<Response>()
		.await
		.map_err(|_| Error::ParseJSON)?;
	let steam_user = if json.response.players.is_empty() {
		return Err(eyre!("empty response from steam"));
	} else {
		json.response.players.remove(0)
	};

	if let Some(avatar_url) = steam_user.avatarfull {
		Ok(avatar_url)
	} else if let Some(avatarmedium) = steam_user.avatarmedium {
		Ok(avatarmedium)
	} else if let Some(avatar) = steam_user.avatar {
		Ok(avatar)
	} else {
		Err(eyre!("user has no avatar"))
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SteamUser {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InnerResponse {
	pub players: Vec<SteamUser>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Response {
	pub response: InnerResponse,
}
