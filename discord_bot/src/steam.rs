//! Steam WebAPI functions.

use {
	crate::Error,
	color_eyre::{eyre::eyre, Result as Eyre},
	serde::Deserialize,
	serde_json::Value as JsonValue,
};

#[tracing::instrument]
pub async fn get_steam_avatar(
	steam_api_key: &str,
	steam_id64: u64,
	client: &gokz_rs::Client,
) -> Eyre<String> {
	let mut response = client
		.get(format!("https://api.steampowered.com/ISteamUser/GetPlayerSummaries/v0002/?key={steam_api_key}&steamids={steam_id64}"))
		.send()
		.await?
		.json::<Response>()
		.await
		.map_err(|_| Error::ParseJSON)
		.map(|response| response.response.players)?;

	let steam_user = if response.is_empty() {
		return Err(eyre!("Empty response from SteamAPI."));
	} else {
		response.remove(0)
	};

	if let Some(avatar_url) = steam_user.avatarfull {
		Ok(avatar_url)
	} else if let Some(avatarmedium) = steam_user.avatarmedium {
		Ok(avatarmedium)
	} else if let Some(avatar) = steam_user.avatar {
		Ok(avatar)
	} else {
		Err(eyre!("Could not find an avatar for `{steam_id64}`."))
	}
}

#[derive(Debug, Clone, Deserialize)]
#[allow(unused)]
struct SteamUser {
	steamid: Option<JsonValue>,
	communityvisibilitystate: Option<JsonValue>,
	profilestate: Option<JsonValue>,
	personaname: Option<JsonValue>,
	commentpermission: Option<JsonValue>,
	profileurl: Option<JsonValue>,
	pub avatar: Option<String>,
	pub avatarmedium: Option<String>,
	pub avatarfull: Option<String>,
	avatarhash: Option<JsonValue>,
	lastlogoff: Option<JsonValue>,
	personastate: Option<JsonValue>,
	realname: Option<JsonValue>,
	primaryclanid: Option<JsonValue>,
	timecreated: Option<JsonValue>,
	personastateflags: Option<JsonValue>,
	loccountrycode: Option<JsonValue>,
}

#[derive(Debug, Clone, Deserialize)]
struct InnerResponse {
	pub players: Vec<SteamUser>,
}

#[derive(Debug, Clone, Deserialize)]
struct Response {
	pub response: InnerResponse,
}
