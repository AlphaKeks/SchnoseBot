use std::error::Error;

use chrono::NaiveDateTime;
use futures::future::join_all;
use regex::Regex;
use reqwest::{Client, StatusCode, Url};
use serde::de::DeserializeOwned;

use crate::structs::*;

pub type APIError = Box<dyn Error>;

#[allow(dead_code)]
pub enum NameOrId {
	Name(String),
	Id(u16),
}

#[allow(dead_code)]
#[derive(Clone)]
pub enum NameOrSteamID {
	Name(String),
	SteamID(String),
}

#[allow(dead_code)]
pub enum ModeInput {
	Raw(String),
	Long(String),
	Short(String),
}

#[allow(dead_code)]
impl ModeInput {
	pub fn to_raw(&self) -> String {
		match self {
			Self::Raw(str) => match str.as_str() {
				"kz_timer" => String::from("kz_timer"),
				"kz_simple" => String::from("kz_simple"),
				"kz_vanilla" => String::from("kz_vanilla"),
				_ => String::new(),
			},
			Self::Long(str) => match str.as_str() {
				"KZTimer" => String::from("kz_timer"),
				"SimpleKZ" => String::from("kz_simple"),
				"Vanilla" => String::from("kz_vanilla"),
				_ => String::new(),
			},
			Self::Short(str) => match str.as_str() {
				"KZT" => String::from("kz_timer"),
				"SKZ" => String::from("kz_simple"),
				"VNL" => String::from("kz_vanilla"),
				_ => String::new(),
			},
		}
	}

	pub fn to_long(&self) -> String {
		match self {
			Self::Raw(str) => match str.as_str() {
				"kz_timer" => String::from("KZTimer"),
				"kz_simple" => String::from("SimpleKZ"),
				"kz_vanilla" => String::from("Vanilla"),
				_ => String::new(),
			},
			Self::Long(str) => match str.as_str() {
				"KZTimer" => String::from("KZTimer"),
				"SimpleKZ" => String::from("SimpleKZ"),
				"Vanilla" => String::from("Vanilla"),
				_ => String::new(),
			},
			Self::Short(str) => match str.as_str() {
				"KZT" => String::from("KZTimer"),
				"SKZ" => String::from("SimpleKZ"),
				"VNL" => String::from("Vanilla"),
				_ => String::new(),
			},
		}
	}

	pub fn to_short(&self) -> String {
		match self {
			Self::Raw(str) => match str.as_str() {
				"kz_timer" => String::from("KZT"),
				"kz_simple" => String::from("SKZ"),
				"kz_vanilla" => String::from("VNL"),
				_ => String::new(),
			},
			Self::Long(str) => match str.as_str() {
				"KZTimer" => String::from("KZT"),
				"SimpleKZ" => String::from("SKZ"),
				"Vanilla" => String::from("VNL"),
				_ => String::new(),
			},
			Self::Short(str) => match str.as_str() {
				"KZT" => String::from("KZT"),
				"SKZ" => String::from("SKZ"),
				"VNL" => String::from("VNL"),
				_ => String::new(),
			},
		}
	}
}

/* GlobalAPI functions */
// base function that gets called by all other
// functions
#[allow(dead_code)]
async fn api_request<T>(
	route: &str,
	params: Vec<(&str, &str)>,
	client: &Client,
) -> Result<T, APIError>
where
	T: DeserializeOwned,
{
	let url = format!("https://kztimerglobal.com/api/v2.0/{route}");
	let url = match Url::parse_with_params(&url, &params) {
		Ok(url) => url,
		_ => return Err("Invalid URL.".into()),
	};

	let request = client.get(url).send().await?;

	match request.status() {
		StatusCode::OK => match request.json::<T>().await {
			Ok(json) => Ok(json),
			_ => Err("GlobalAPI returned invalid data.".into()),
		},
		_ => Err("GlobalAPI request failed.".into()),
	}
}

#[allow(dead_code)]
pub async fn get_maps(client: &Client) -> Result<Vec<Map>, APIError> {
	let params = vec![("is_validated", "true"), ("limit", "9999")];

	api_request::<Vec<Map>>("maps?", params, client).await
}

#[allow(dead_code)]
pub async fn get_map(map: NameOrId, client: &Client) -> Result<Map, APIError> {
	let mut params = vec![("is_validated", "true"), ("limit", "1")];

	let map = match map {
		NameOrId::Name(name) => ("name", name),
		NameOrId::Id(id) => ("id", id.to_string()),
	};

	params.push((map.0, &map.1));

	match api_request::<Vec<Map>>("maps?", params, client).await {
		Ok(mut maps) => {
			if maps.len() > 0 {
				Ok(maps.remove(0))
			} else {
				Err("GlobalAPI returned invalid data.".into())
			}
		}
		Err(err) => return Err(err),
	}
}

#[allow(dead_code)]
pub async fn get_filters(
	map_id: &str,
	course: &str,
	client: &Client,
) -> Result<DisplayFilterCollection, APIError> {
	let params = vec![
		("map_ids", map_id),
		("stages", course),
		("tickrates", "128"),
		("has_teleports", "false"),
		("limit", "9999"),
	];

	match api_request::<Vec<RecordFilter>>("record_filters?", params, client).await {
		Ok(filters) => {
			let mut display_filters = DisplayFilterCollection {
				kzt: DisplayFilter {
					mode: "kz_timer".to_string(),
					display_mode: "KZTimer".to_string(),
					short_mode: "KZT".to_string(),
					mode_id: 200,
					icon: "❌".to_string(),
				},
				skz: DisplayFilter {
					mode: "kz_simple".to_string(),
					display_mode: "SimpleKZ".to_string(),
					short_mode: "SKZ".to_string(),
					mode_id: 201,
					icon: "❌".to_string(),
				},
				vnl: DisplayFilter {
					mode: "kz_vanilla".to_string(),
					display_mode: "Vanilla".to_string(),
					short_mode: "VNL".to_string(),
					mode_id: 202,
					icon: "❌".to_string(),
				},
			};
			for filter in filters {
				match filter.mode_id {
					200 => display_filters.kzt.icon = "✅".to_string(),
					201 => display_filters.skz.icon = "✅".to_string(),
					202 => display_filters.vnl.icon = "✅".to_string(),
					_ => (),
				}
			}
			return Ok(display_filters);
		}
		Err(err) => Err(err),
	}
}

#[allow(dead_code)]
pub async fn get_filter_dist(
	mode_id: &str,
	runtype: &str,
	client: &Client,
) -> Result<Vec<RecordFilter>, APIError> {
	let params = vec![
		("stages", "0"),
		("mode_ids", mode_id),
		("tickrates", "128"),
		("has_teleports", runtype),
		("limit", "9999"),
	];

	api_request::<Vec<RecordFilter>>("record_filters?", params, client).await
}

#[allow(dead_code)]
pub async fn get_modes(client: &Client) -> Result<Vec<Mode>, APIError> {
	api_request("modes?", vec![], client).await
}

#[allow(dead_code)]
pub async fn get_mode(mode: NameOrId, client: &Client) -> Result<Mode, APIError> {
	let mut path = String::from("modes/");
	match mode {
		NameOrId::Name(name) => path = format!("{}/name/{}", path, name),
		NameOrId::Id(id) => path = format!("{}/id/{}", path, id),
	};

	match api_request::<Vec<Mode>>(path.as_str(), vec![], client).await {
		Ok(mut modes) => {
			if modes.len() > 0 {
				Ok(modes.remove(0))
			} else {
				Err("GlobalAPI returned invalid data.".into())
			}
		}
		Err(err) => Err(err),
	}
}

#[allow(dead_code)]
pub async fn get_player(player: NameOrSteamID, client: &Client) -> Result<Player, APIError> {
	let mut params = vec![("limit", "1")];
	let player = match player {
		NameOrSteamID::Name(name) => ("name", name),
		NameOrSteamID::SteamID(steam_id) => ("steam_id", steam_id),
	};

	params.push((player.0, &player.1));

	match api_request::<Vec<Player>>("players?", params, client).await {
		Ok(mut players) => {
			if players.len() > 0 {
				Ok(players.remove(0))
			} else {
				Err("GlobalAPI returned invalid data.".into())
			}
		}
		Err(err) => Err(err),
	}
}

#[allow(dead_code)]
pub async fn get_wr(
	map: NameOrId,
	course: &str,
	mode_name: &str,
	runtype: &str,
	client: &Client,
) -> Result<Record, APIError> {
	let mut params = vec![
		("tickrate", "128"),
		("stage", course),
		("modes_list_string", mode_name),
		("has_teleports", runtype),
		("limit", "1"),
	];

	let map = match map {
		NameOrId::Name(name) => ("map_name", name),
		NameOrId::Id(id) => ("map_id", id.to_string()),
	};

	params.push((map.0, &map.1));

	match api_request::<Vec<Record>>("records/top?", params, client).await {
		Ok(mut records) => {
			if records.len() > 0 {
				Ok(records.remove(0))
			} else {
				Err("GlobalAPI returned invalid data.".into())
			}
		}
		Err(err) => Err(err),
	}
}

#[allow(dead_code)]
pub async fn get_maptop(
	map: NameOrId,
	mode_name: &str,
	course: &str,
	runtype: &str,
	client: &Client,
) -> Result<Vec<Record>, APIError> {
	let mut params = vec![
		("tickrate", "128"),
		("stage", course),
		("modes_list_string", mode_name),
		("has_teleports", runtype),
		("limit", "100"),
	];

	let map = match map {
		NameOrId::Name(name) => ("map_name", name),
		NameOrId::Id(id) => ("map_id", id.to_string()),
	};

	params.push((map.0, &map.1));

	api_request::<Vec<Record>>("records/top?", params, client).await
}

#[allow(dead_code)]
pub async fn get_top(
	mode: NameOrId,
	stages: Vec<&str>,
	runtype: &str,
	client: &Client,
) -> Result<Vec<Place>, APIError> {
	let mut params = vec![
		("tickrates", "128"),
		("has_teleports", runtype),
		("limit", "100"),
	];

	let mode = match mode {
		NameOrId::Name(name) => match name.as_str() {
			"kz_timer" => ("mode_ids", 200.to_string()),
			"kz_simple" => ("mode_ids", 201.to_string()),
			"kz_vanilla" => ("mode_ids", 202.to_string()),
			// kzt as fallback
			_ => ("mode_ids", 200.to_string()),
		},
		NameOrId::Id(id) => ("mode_ids", id.to_string()),
	};

	params.push((mode.0, &mode.1));

	let mut path = String::from("records/top/world_records?");
	for i in stages {
		path.push_str(format!("stages={i}&").as_str());
	}

	api_request::<Vec<Place>>(path.as_str(), params, client).await
}

#[allow(dead_code)]
pub async fn get_pb(
	player: NameOrSteamID,
	map: NameOrId,
	course: &str,
	mode_name: &str,
	runtype: &str,
	client: &Client,
) -> Result<Record, APIError> {
	let mut params = vec![
		("tickrates", "128"),
		("stage", course),
		("modes_list_string", mode_name),
		("has_teleports", runtype),
		("limit", "1"),
	];

	let player = match player {
		NameOrSteamID::Name(name) => ("player_name", name),
		NameOrSteamID::SteamID(steam_id) => ("steam_id", steam_id),
	};

	params.push((player.0, &player.1));

	let map = match map {
		NameOrId::Name(name) => ("map_name", name),
		NameOrId::Id(id) => ("map_id", id.to_string()),
	};

	params.push((map.0, &map.1));

	match api_request::<Vec<Record>>("records/top?", params, client).await {
		Ok(mut records) => {
			if records.len() > 0 {
				Ok(records.remove(0))
			} else {
				Err("GlobalAPI returned invalid data.".into())
			}
		}
		Err(err) => Err(err),
	}
}

#[allow(dead_code)]
pub async fn get_times(
	player: NameOrSteamID,
	mode_name: &str,
	runtype: &str,
	client: &Client,
) -> Result<Vec<Record>, APIError> {
	let mut params = vec![
		("tickrates", "128"),
		("stage", "0"),
		("modes_list_string", mode_name),
		("has_teleports", runtype),
		("limit", "9999"),
	];

	let player = match player {
		NameOrSteamID::Name(name) => ("player_name", name),
		NameOrSteamID::SteamID(steam_id) => ("steam_id", steam_id),
	};

	params.push((player.0, &player.1));

	api_request::<Vec<Record>>("records/top?", params, client).await
}

#[allow(dead_code)]
pub async fn get_recent(player: NameOrSteamID, client: Client) -> Result<Record, APIError> {
	let mut player_vars = vec![];
	let mut client_vars = vec![];
	for _ in 0..5 {
		player_vars.push(player.clone());
		client_vars.push(client.clone());
	}

	let requests1 = vec![
		get_times(player_vars.remove(0), "kz_timer", "true", &client_vars[0]),
		get_times(player_vars.remove(0), "kz_timer", "false", &client_vars[1]),
		get_times(player_vars.remove(0), "kz_simple", "true", &client_vars[2]),
	];

	let mut result1 = join_all(requests1).await;

	let requests2 = vec![
		get_times(player_vars.remove(0), "kz_simple", "false", &client_vars[3]),
		get_times(player_vars.remove(0), "kz_vanilla", "true", &client_vars[4]),
		get_times(player, "kz_vanilla", "false", &client),
	];

	let mut result2 = join_all(requests2).await;

	let mut result = vec![];
	result.append(&mut result1);
	result.append(&mut result2);

	let mut records: Vec<Record> = vec![];
	for i in result {
		match i {
			Ok(mut data) => records.append(&mut data),
			_ => (),
		}
	}

	println!("{} results", records.len());

	if records.len() < 1 {
		return Err("This player has no recent times.".into());
	} else {
		let mut recent: (i64, &Record) = (0, &records[0]);

		for i in 1..records.len() {
			let date =
				NaiveDateTime::parse_from_str(records[i].created_on.as_str(), "%Y-%m-%dT%H:%M:%S")?;
			if date.timestamp() > recent.0 {
				recent = (date.timestamp(), &records[i]);
			}
		}

		Ok(recent.1.to_owned())
	}
}

#[allow(dead_code)]
pub async fn get_place(record: Record, client: &Client) -> Result<u16, APIError> {
	api_request::<u16>(
		format!("records/place/{}", record.id).as_str(),
		vec![],
		client,
	)
	.await
}

/* Utility functions */
#[allow(dead_code)]
pub async fn api_status(client: &Client) -> Result<APIStatusShort, APIError> {
	let url = format!("https://status.global-api.com/api/v2/summary.json");
	let request = client.get(url).send().await?;

	match request.status() {
		StatusCode::OK => match request.json::<APIStatus>().await {
			Ok(mut json) => Ok(APIStatusShort {
				status: json.status.description,
				frontend: json.components.remove(0).status,
				backend: json.components.remove(0).status,
			}),
			_ => Err("GlobalAPI returned invalid data.".into()),
		},
		_ => Err("GlobalAPI request failed.".into()),
	}
}

#[allow(dead_code)]
pub fn is_steamid(input: &str) -> bool {
	let regex = Regex::new(r"STEAM_[0-1]:[0-1]:[0-9]+");
	match regex {
		Ok(r) => match r.find(&input) {
			Some(_) => true,
			None => false,
		},
		Err(_) => false,
	}
}

#[allow(dead_code)]
pub async fn to_steamid(player_identifier: &str) -> Result<NameOrSteamID, String> {
	if is_steamid(&player_identifier) {
		Ok(NameOrSteamID::SteamID(player_identifier.to_string()))
	} else {
		let client = Client::new();
		match get_player(NameOrSteamID::Name(player_identifier.to_string()), &client).await {
			Ok(player) => match player.steam_id {
				Some(steam_id) => Ok(NameOrSteamID::SteamID(steam_id)),
				None => Err("Couldn't convert to steamID.".to_string()),
			},
			_ => Err("Couldn't convert to steamID.".to_string()),
		}
	}
}

#[allow(dead_code)]
pub async fn get_mapcycle(client: &Client) -> Result<Vec<String>, APIError> {
	let url = format!("https://maps.cawkz.net/mapcycles/gokz.txt");
	let request = client.get(url).send().await?;

	match request.status() {
		StatusCode::OK => match request.json::<String>().await {
			Ok(res) => {
				let mut mapcycle = vec![];
				for map in res.split("\r\n").into_iter() {
					mapcycle.push(map.to_string());
				}
				Ok(mapcycle)
			}
			_ => Err("GlobalAPI returned invalid data.".into()),
		},
		_ => Err("GlobalAPI request failed.".into()),
	}
}

#[allow(dead_code)]
pub async fn validate_map(map_name: &str, map_list: Vec<Map>) -> Result<Map, APIError> {
	for map in map_list {
		if map.name.contains(map_name.to_lowercase().as_str()) {
			return Ok(map);
		}
	}
	Err("The provided map is not global.".into())
}

#[allow(dead_code)]
pub async fn get_tier(map_name: &str, map_list: Vec<Map>) -> Result<u8, APIError> {
	for map in map_list {
		if map.name.contains(map_name.to_lowercase().as_str()) {
			return Ok(map.difficulty);
		}
	}
	Err("The provided map is not global.".into())
}

/* KZ:GO API functions */
#[allow(dead_code)]
pub async fn get_maps_kzgo(client: &Client) -> Result<Vec<KZGOMap>, APIError> {
	let url = format!("https://kzgo.eu/api/maps");
	let request = client.get(url).send().await?;

	match request.status() {
		StatusCode::OK => match request.json::<Vec<KZGOMap>>().await {
			Ok(json) => Ok(json),
			_ => Err("KZGO API returned invalid data.".into()),
		},
		_ => Err("KZGO API request failed.".into()),
	}
}

#[allow(dead_code)]
pub async fn get_map_kzgo(map_name: &str, client: &Client) -> Result<KZGOMap, APIError> {
	let url = format!("https://kzgo.eu/api/maps/{}", map_name);
	let request = client.get(url).send().await?;

	match request.status() {
		StatusCode::OK => match request.json::<KZGOMap>().await {
			Ok(json) => Ok(json),
			_ => Err("KZGO API returned invalid data.".into()),
		},
		_ => Err("KZGO API request failed.".into()),
	}
}