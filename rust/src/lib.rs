extern crate wasm_bindgen;

use std::{collections::HashMap, str::FromStr};

use chrono::NaiveDateTime;
use futures::future::join_all;
use reqwest::{Client, StatusCode};
use serde::Serialize;
use wasm_bindgen::prelude::*;

mod core;
mod structs;

use crate::core::*;
use crate::structs::*;

// type JSResult = Result<String, String>;
type JSResult = String;

#[wasm_bindgen]
pub async fn apistatus_wasm() -> JSResult {
	let client = Client::new();

	let result = match crate::core::api_status(&client).await {
		Ok(data) => data,
		Err(err) => return err.to_string(),
	};

	match serde_json::to_string_pretty(&result) {
		Ok(json) => json,
		_ => "GlobalAPI returned invalid data.".to_string(),
	}
}

#[wasm_bindgen]
pub async fn map_wasm(map_name: String) -> JSResult {
	#[derive(Serialize)]
	struct EmbedFilter {
		name: String,
		value: String,
		inline: bool,
	}

	#[derive(Serialize)]
	struct GetMapResult {
		title: String,
		url: String,
		thumbnail: String,
		tier: u8,
		mappers: Vec<String>,
		bonuses: u8,
		date: String,
		filters: [EmbedFilter; 3],
	}

	let client = Client::new();

	let global_maps = match get_maps(&client).await {
		Ok(maps) => maps,
		Err(err) => return err.to_string(),
	};

	let map = match validate_map(&map_name, global_maps).await {
		Ok(map) => map,
		Err(err) => return err.to_string(),
	};

	let kzgo_map = match crate::core::get_map_kzgo(&map.name, &client).await {
		Ok(data) => data,
		Err(err) => return err.to_string(),
	};

	let mut mappers = vec![];
	for i in 0..kzgo_map.mapper_names.len() {
		mappers.push(format!(
			"[{}](https://steamcommunity.com/profiles/{})",
			kzgo_map.mapper_names[i], kzgo_map.mapper_ids[i]
		));
	}

	let filters = match crate::core::get_filters(&map.id.to_string(), "0", &client).await {
		Ok(data) => data,
		Err(err) => return err.to_string(),
	};

	let date = match NaiveDateTime::from_str(&map.updated_on) {
		Ok(str) => str,
		_ => return "Error parsing date.".to_string(),
	};

	let embed_filters = [
		EmbedFilter {
			name: filters.kzt.short_mode,
			value: filters.kzt.icon,
			inline: true,
		},
		EmbedFilter {
			name: filters.skz.short_mode,
			value: filters.skz.icon,
			inline: true,
		},
		EmbedFilter {
			name: filters.vnl.short_mode,
			value: filters.vnl.icon,
			inline: true,
		},
	];

	let result = GetMapResult {
		title: map.name.clone(),
		url: format!("https://kzgo.eu/maps/{}", map.name).to_string(),
		thumbnail: format!(
			"https://raw.githubusercontent.com/KZGlobalTeam/map-images/master/images/{}.jpg",
			map.name
		)
		.to_string(),
		tier: map.difficulty,
		mappers,
		bonuses: kzgo_map.bonuses,
		date: format!("<t:{}:d>", date.timestamp()).to_string(),
		filters: embed_filters,
	};

	match serde_json::to_string_pretty(&result) {
		Ok(str) => str,
		Err(err) => return err.to_string(),
	}
}

#[wasm_bindgen]
pub async fn wr_wasm(map_name: String, mode_name: String) -> JSResult {
	let client = Client::new();

	let global_maps = match get_maps(&client).await {
		Ok(maps) => maps,
		Err(err) => return err.to_string(),
	};

	let map = match validate_map(&map_name, global_maps).await {
		Ok(map) => map,
		Err(err) => return err.to_string(),
	};

	let wrs = vec![
		get_wr(
			NameOrId::Name(map.name.clone()),
			"0",
			&mode_name,
			"true",
			&client,
		),
		get_wr(NameOrId::Name(map.name), "0", &mode_name, "false", &client),
	];

	let wr_data = join_all(wrs.into_iter()).await;

	let mut result = vec![];
	for wr in wr_data {
		if let Ok(data) = wr {
			if let Ok(json) = serde_json::to_string_pretty(&data) {
				result.push(json);
			} else {
				result.push("null".to_string());
			}
		} else {
			result.push("null".to_string());
		}
	}

	if result.len() > 0 {
		match serde_json::to_string_pretty(&result) {
			Ok(result) => result,
			_ => "Failed to serialize data.".to_string(),
		}
	} else {
		"GlobalAPI returned invalid data.".to_string()
	}
}

#[wasm_bindgen]
pub async fn pb_wasm(map_name: String, mode_name: String, player_identifier: String) -> JSResult {
	let player = match to_steamid(&player_identifier).await {
		Ok(player) => player,
		_ => NameOrSteamID::Name(player_identifier),
	};

	let client = Client::new();

	let global_maps = match get_maps(&client).await {
		Ok(maps) => maps,
		Err(err) => return err.to_string(),
	};

	let map = match validate_map(&map_name, global_maps).await {
		Ok(map) => map,
		Err(err) => return err.to_string(),
	};

	let requests = vec![
		get_pb(
			player.clone(),
			NameOrId::Name(map.name.clone()),
			"0",
			&mode_name,
			"true",
			&client,
		),
		get_pb(
			player,
			NameOrId::Name(map.name),
			"0",
			&mode_name,
			"false",
			&client,
		),
	];

	let pb_data = join_all(requests.into_iter()).await;

	let mut result = vec![];
	for pb in pb_data {
		if let Ok(data) = pb {
			if let Ok(json) = serde_json::to_string_pretty(&data) {
				result.push(json);
			} else {
				result.push("null".to_string());
			}
		} else {
			result.push("null".to_string());
		}
	}

	if result.len() > 0 {
		match serde_json::to_string_pretty(&result) {
			Ok(result) => result,
			_ => "Failed to serialize data.".to_string(),
		}
	} else {
		"GlobalAPI returned invalid data.".to_string()
	}
}

#[wasm_bindgen]
pub async fn bwr_wasm(map_name: String, course: u32, mode_name: String) -> JSResult {
	let course = course.to_string();
	let client = Client::new();

	let global_maps = match get_maps(&client).await {
		Ok(maps) => maps,
		Err(err) => return err.to_string(),
	};

	let map = match validate_map(&map_name, global_maps).await {
		Ok(map) => map,
		Err(err) => return err.to_string(),
	};

	let wrs = vec![
		get_wr(
			NameOrId::Name(map.name.clone()),
			&course,
			&mode_name,
			"true",
			&client,
		),
		get_wr(
			NameOrId::Name(map.name),
			&course,
			&mode_name,
			"false",
			&client,
		),
	];

	let wr_data = join_all(wrs.into_iter()).await;

	let mut result = vec![];
	for wr in wr_data {
		if let Ok(data) = wr {
			if let Ok(json) = serde_json::to_string_pretty(&data) {
				result.push(json);
			} else {
				result.push("null".to_string());
			}
		} else {
			result.push("null".to_string());
		}
	}

	if result.len() > 0 {
		match serde_json::to_string_pretty(&result) {
			Ok(result) => result,
			_ => "Failed to serialize data.".to_string(),
		}
	} else {
		"GlobalAPI returned invalid data.".to_string()
	}
}

#[wasm_bindgen]
pub async fn bpb_wasm(
	map_name: String,
	course: u32,
	mode_name: String,
	player_identifier: String,
) -> JSResult {
	let player = match to_steamid(&player_identifier).await {
		Ok(player) => player,
		_ => NameOrSteamID::Name(player_identifier),
	};

	let course = course.to_string();
	let client = Client::new();

	let global_maps = match get_maps(&client).await {
		Ok(maps) => maps,
		Err(err) => return err.to_string(),
	};

	let map = match validate_map(&map_name, global_maps).await {
		Ok(map) => map,
		Err(err) => return err.to_string(),
	};

	let requests = vec![
		get_pb(
			player.clone(),
			NameOrId::Name(map.name.clone()),
			&course,
			&mode_name,
			"true",
			&client,
		),
		get_pb(
			player,
			NameOrId::Name(map.name),
			&course,
			&mode_name,
			"false",
			&client,
		),
	];

	let pb_data = join_all(requests.into_iter()).await;

	let mut result = vec![];
	for pb in pb_data {
		if let Ok(data) = pb {
			if let Ok(json) = serde_json::to_string_pretty(&data) {
				result.push(json);
			} else {
				result.push("null".to_string());
			}
		} else {
			result.push("null".to_string());
		}
	}

	if result.len() > 0 && (result[0].len() > 2 && result[1].len() > 2) {
		match serde_json::to_string_pretty(&result) {
			Ok(result) => result,
			_ => "Failed to serialize data.".to_string(),
		}
	} else {
		return "GlobalAPI returned invalid data.".to_string();
	}
}

#[wasm_bindgen]
pub async fn recent_wasm(player_identifier: String) -> JSResult {
	let player = match to_steamid(&player_identifier).await {
		Ok(player) => player,
		_ => NameOrSteamID::Name(player_identifier),
	};

	let client = Client::new();
	let result = match get_recent(player, client).await {
		Ok(data) => data,
		Err(err) => return err.to_string(),
	};

	match serde_json::to_string_pretty(&result) {
		Ok(json) => json,
		_ => "GlobalAPI returned invalid data.".to_string(),
	}
}

#[wasm_bindgen]
pub async fn unfinished_wasm(
	tier: Option<u8>,
	mode_name: String,
	runtype: bool,
	player_identifier: String,
) -> JSResult {
	let player = match to_steamid(&player_identifier).await {
		Ok(player) => player,
		_ => NameOrSteamID::Name(player_identifier),
	};

	let mode_id = match mode_name.as_str() {
		"kz_timer" => "200",
		"kz_simple" => "201",
		"kz_vanilla" => "202",
		_ => "200",
	};

	let client = Client::new();

	let completed_maps = get_times(player, &mode_name, &runtype.to_string(), &client).await;
	let doable_maps = match get_filter_dist(mode_id, &runtype.to_string(), &client).await {
		Ok(maps) => maps,
		_ => return "GlobalAPI returned invalid data.".to_string(),
	};

	let mut comp_ids = vec![];
	let mut uncomp_ids = vec![];

	if let Ok(maps) = completed_maps {
		for map in maps {
			comp_ids.push(map.map_id);
		}
	}

	for map in doable_maps {
		if !comp_ids.contains(&map.map_id) {
			uncomp_ids.push(map.map_id);
		}
	}

	let global_maps = match get_maps(&client).await {
		Ok(maps) => maps,
		_ => return "GlobalAPI returned invalid data.".to_string(),
	};

	let mut uncompleted_maps = vec![];
	for map in global_maps {
		if uncomp_ids.contains(&map.id.into())
			&& (match tier {
				Some(tier) => map.difficulty == tier,
				None => true,
			}) && (if runtype {
			!map.name.starts_with("kzpro_")
		} else {
			true
		}) {
			uncompleted_maps.push(map.name);
		}
	}

	if uncompleted_maps.len() == 0 {
		return "Congrats, you have no maps left to complete! Good Job 🎉".to_string();
	}

	let mut text = String::new();
	for i in 0..uncompleted_maps.len() {
		if i == 10 {
			text.push_str(format!("...{} more 😔", uncompleted_maps.len() - 10).as_str());
			break;
		}
		text.push_str(format!("> {}\n", uncompleted_maps[i]).as_str());
	}

	text
}

#[wasm_bindgen]
pub async fn profile_wasm(player_identifier: String, mode_name: String) -> JSResult {
	let player = match to_steamid(&player_identifier).await {
		Ok(player) => player,
		_ => NameOrSteamID::Name(player_identifier),
	};

	#[derive(Serialize)]
	struct PlayerProfile {
		tp_points: u32,
		pro_points: u32,
		tp_recs: u32,
		pro_recs: u32,
		tp_perc: f32,
		pro_perc: f32,
		tp_runs: [u32; 8],
		pro_runs: [u32; 8],
		steamid64: String,
		steam_id: Option<String>,
		is_banned: bool,
		total_records: u32,
		name: Option<String>,
		rank: String,
		doable: [[u32; 8]; 2],
		bars: Vec<Vec<String>>,
	}

	let client = Client::new();

	let mut player = match get_player(player, &client).await {
		Ok(data) => PlayerProfile {
			tp_points: 0,
			pro_points: 0,
			tp_recs: 0,
			pro_recs: 0,
			tp_perc: 0.0,
			pro_perc: 0.0,
			tp_runs: [0; 8],
			pro_runs: [0; 8],
			steamid64: data.steamid64,
			steam_id: data.steam_id,
			is_banned: data.is_banned,
			total_records: data.total_records,
			name: data.name,
			rank: String::from("Unknown"),
			doable: [[0; 8]; 2],
			bars: {
				let mut arr = vec![];
				for _ in 0..2 {
					let mut temp = vec![];
					for _ in 0..7 {
						temp.push(String::from(""));
					}
					arr.push(temp);
				}
				arr
			},
		},
		Err(err) => return err.to_string(),
	};

	let global_maps = match get_maps(&client).await {
		Ok(maps) => maps,
		Err(err) => return err.to_string(),
	};

	let mut tiers = vec![HashMap::new(), HashMap::new()];
	for i in 0..global_maps.len() {
		let names = (
			global_maps[i].name.to_owned(),
			global_maps[i].name.to_owned(),
		);
		let map_tiers = (
			global_maps[i].difficulty.to_owned(),
			global_maps[i].difficulty.to_owned(),
		);
		tiers[0].insert(names.0, map_tiers.0);
		tiers[1].insert(names.1, map_tiers.1);
	}

	let ids = (player.steam_id.clone(), player.steam_id.clone());
	let names = (player.name.clone(), player.name.clone());

	let times = vec![
		get_times(
			match ids.0 {
				Some(steam_id) => NameOrSteamID::SteamID(steam_id),
				None => match names.0 {
					Some(name) => NameOrSteamID::Name(name),
					None => NameOrSteamID::Name(String::from("")),
				},
			},
			&mode_name,
			"true",
			&client,
		),
		get_times(
			match ids.1 {
				Some(steam_id) => NameOrSteamID::SteamID(steam_id),
				None => match names.1 {
					Some(name) => NameOrSteamID::Name(name),
					None => NameOrSteamID::Name(String::from("")),
				},
			},
			&mode_name,
			"false",
			&client,
		),
	];

	let mut results = join_all(times.into_iter()).await;

	let tp = match results[0] {
		Ok(_) => {
			let h = results.remove(0);
			if let Ok(data) = h {
				Some(data)
			} else {
				None
			}
		}
		_ => None,
	};

	let pro = match results[0] {
		Ok(_) => {
			let h = results.remove(0);
			if let Ok(data) = h {
				Some(data)
			} else {
				None
			}
		}
		_ => None,
	};

	let lens = (tp.clone(), pro.clone());

	let longest = match lens.0 {
		Some(data) => {
			let plen = match lens.1 {
				Some(data) => data.len(),
				None => 0,
			};

			if data.len() > plen {
				data.len()
			} else {
				plen
			}
		}
		None => 0,
	};

	for i in 0..longest {
		if let Some(ref tp) = tp {
			if tp.len() > i {
				if tiers[0].contains_key(&tp[i].map_name) {
					player.tp_points += tp[i].points;
					player.tp_runs[7] += 1;

					match tiers[0].get(&tp[i].map_name) {
						Some(1) => player.tp_runs[0] += 1,
						Some(2) => player.tp_runs[1] += 1,
						Some(3) => player.tp_runs[2] += 1,
						Some(4) => player.tp_runs[3] += 1,
						Some(5) => player.tp_runs[4] += 1,
						Some(6) => player.tp_runs[5] += 1,
						Some(7) => player.tp_runs[6] += 1,
						_ => (),
					}

					if tp[i].points == 1000 {
						player.tp_recs += 1;
					}

					tiers[0].remove_entry(&tp[i].map_name);
				}
			}
		}

		if let Some(ref pro) = pro {
			if pro.len() > i {
				if tiers[1].contains_key(&pro[i].map_name) {
					player.pro_points += pro[i].points;
					player.pro_runs[7] += 1;

					match tiers[1].get(&pro[i].map_name) {
						Some(1) => player.pro_runs[0] += 1,
						Some(2) => player.pro_runs[1] += 1,
						Some(3) => player.pro_runs[2] += 1,
						Some(4) => player.pro_runs[3] += 1,
						Some(5) => player.pro_runs[4] += 1,
						Some(6) => player.pro_runs[5] += 1,
						Some(7) => player.pro_runs[6] += 1,
						_ => (),
					}

					if pro[i].points == 1000 {
						player.pro_recs += 1;
					}

					tiers[1].remove_entry(&pro[i].map_name);
				}
			}
		}
	}

	let total_points = &player.tp_points + &player.pro_points;
	match mode_name.as_str() {
		"kz_timer" => {
			if total_points >= 1_000_000 {
				player.rank = "Legend".to_string();
			} else if total_points >= 800_000 {
				player.rank = "Master".to_string();
			} else if total_points >= 600_000 {
				player.rank = "Pro".to_string();
			} else if total_points >= 400_000 {
				player.rank = "Semipro".to_string();
			} else if total_points >= 250_000 {
				player.rank = "Expert+".to_string();
			} else if total_points >= 230_000 {
				player.rank = "Expert".to_string();
			} else if total_points >= 200_000 {
				player.rank = "Expert-".to_string();
			} else if total_points >= 150_000 {
				player.rank = "Skilled+".to_string();
			} else if total_points >= 120_000 {
				player.rank = "Skilled".to_string();
			} else if total_points >= 100_000 {
				player.rank = "Skilled-".to_string();
			} else if total_points >= 80_000 {
				player.rank = "Regular+".to_string();
			} else if total_points >= 70_000 {
				player.rank = "Regular".to_string();
			} else if total_points >= 60_000 {
				player.rank = "Regular-".to_string();
			} else if total_points >= 40_000 {
				player.rank = "Casual+".to_string();
			} else if total_points >= 30_000 {
				player.rank = "Casual".to_string();
			} else if total_points >= 20_000 {
				player.rank = "Casual-".to_string();
			} else if total_points >= 10_000 {
				player.rank = "Amateur+".to_string();
			} else if total_points >= 5_000 {
				player.rank = "Amateur".to_string();
			} else if total_points >= 2_000 {
				player.rank = "Amateur-".to_string();
			} else if total_points >= 1_000 {
				player.rank = "Beginner+".to_string();
			} else if total_points >= 500 {
				player.rank = "Beginner".to_string();
			} else if total_points > 0 {
				player.rank = "Beginner-".to_string();
			} else {
				player.rank = "New".to_string();
			}
		}
		"kz_simple" => {
			if total_points >= 800_000 {
				player.rank = "Legend".to_string();
			} else if total_points >= 500_000 {
				player.rank = "Master".to_string();
			} else if total_points >= 400_000 {
				player.rank = "Pro".to_string();
			} else if total_points >= 300_000 {
				player.rank = "Semipro".to_string();
			} else if total_points >= 250_000 {
				player.rank = "Expert+".to_string();
			} else if total_points >= 230_000 {
				player.rank = "Expert".to_string();
			} else if total_points >= 200_000 {
				player.rank = "Expert-".to_string();
			} else if total_points >= 150_000 {
				player.rank = "Skilled+".to_string();
			} else if total_points >= 120_000 {
				player.rank = "Skilled".to_string();
			} else if total_points >= 100_000 {
				player.rank = "Skilled-".to_string();
			} else if total_points >= 80_000 {
				player.rank = "Regular+".to_string();
			} else if total_points >= 70_000 {
				player.rank = "Regular".to_string();
			} else if total_points >= 60_000 {
				player.rank = "Regular-".to_string();
			} else if total_points >= 40_000 {
				player.rank = "Casual+".to_string();
			} else if total_points >= 30_000 {
				player.rank = "Casual".to_string();
			} else if total_points >= 20_000 {
				player.rank = "Casual-".to_string();
			} else if total_points >= 10_000 {
				player.rank = "Amateur+".to_string();
			} else if total_points >= 5_000 {
				player.rank = "Amateur".to_string();
			} else if total_points >= 2_000 {
				player.rank = "Amateur-".to_string();
			} else if total_points >= 1_000 {
				player.rank = "Beginner+".to_string();
			} else if total_points >= 500 {
				player.rank = "Beginner".to_string();
			} else if total_points > 0 {
				player.rank = "Beginner-".to_string();
			} else {
				player.rank = "New".to_string();
			}
		}
		"kz_vanilla" => {
			if total_points >= 600_000 {
				player.rank = "Legend".to_string();
			} else if total_points >= 400_000 {
				player.rank = "Master".to_string();
			} else if total_points >= 300_000 {
				player.rank = "Pro".to_string();
			} else if total_points >= 250_000 {
				player.rank = "Semipro".to_string();
			} else if total_points >= 200_000 {
				player.rank = "Expert+".to_string();
			} else if total_points >= 180_000 {
				player.rank = "Expert".to_string();
			} else if total_points >= 160_000 {
				player.rank = "Expert-".to_string();
			} else if total_points >= 140_000 {
				player.rank = "Skilled+".to_string();
			} else if total_points >= 120_000 {
				player.rank = "Skilled".to_string();
			} else if total_points >= 100_000 {
				player.rank = "Skilled-".to_string();
			} else if total_points >= 80_000 {
				player.rank = "Regular+".to_string();
			} else if total_points >= 70_000 {
				player.rank = "Regular".to_string();
			} else if total_points >= 60_000 {
				player.rank = "Regular-".to_string();
			} else if total_points >= 40_000 {
				player.rank = "Casual+".to_string();
			} else if total_points >= 30_000 {
				player.rank = "Casual".to_string();
			} else if total_points >= 20_000 {
				player.rank = "Casual-".to_string();
			} else if total_points >= 10_000 {
				player.rank = "Amateur+".to_string();
			} else if total_points >= 5_000 {
				player.rank = "Amateur".to_string();
			} else if total_points >= 2_000 {
				player.rank = "Amateur-".to_string();
			} else if total_points >= 1_000 {
				player.rank = "Beginner+".to_string();
			} else if total_points >= 500 {
				player.rank = "Beginner".to_string();
			} else if total_points > 0 {
				player.rank = "Beginner-".to_string();
			} else {
				player.rank = "New".to_string();
			}
		}
		_ => (),
	}

	let doable_request = match client
		.get(format!("https://kzgo.eu/api/completions/{}", mode_name))
		.send()
		.await
	{
		Ok(data) => match data.status() {
			StatusCode::OK => match data.json::<CompletionStats>().await {
				Ok(json) => json,
				_ => return "Serialization failed.".to_string(),
			},
			_ => return "KZGO API Request failed.".to_string(),
		},
		Err(err) => return err.to_string(),
	};

	player.doable = [
		[
			doable_request.tp.one,
			doable_request.tp.two,
			doable_request.tp.three,
			doable_request.tp.four,
			doable_request.tp.five,
			doable_request.tp.six,
			doable_request.tp.seven,
			doable_request.tp.total,
		],
		[
			doable_request.pro.one,
			doable_request.pro.two,
			doable_request.pro.three,
			doable_request.pro.four,
			doable_request.pro.five,
			doable_request.pro.six,
			doable_request.pro.seven,
			doable_request.pro.total,
		],
	];

	if player.tp_runs[7] > 0 {
		player.tp_perc = (player.tp_runs[7] as f32 / player.doable[0][7] as f32) * 100.0;
	}
	if player.pro_runs[7] > 0 {
		player.pro_perc = (player.pro_runs[7] as f32 / player.doable[1][7] as f32) * 100.0;
	}

	for i in 0..7 {
		let amount_of_bars = (player.tp_runs[i] as f32 / player.doable[0][i] as f32) * 10.0;

		for _ in 0..(amount_of_bars as u32) {
			player.bars[0][i].push_str("█");
		}

		for _ in 0..(10 - amount_of_bars as u32) {
			player.bars[0][i].push_str("░");
		}
	}

	for i in 0..7 {
		let amount_of_bars = (player.pro_runs[i] as f32 / player.doable[1][i] as f32) * 10.0;

		for _ in 0..(amount_of_bars as u32) {
			player.bars[1][i].push_str("█");
		}

		for _ in 0..(10 - amount_of_bars as u32) {
			player.bars[1][i].push_str("░");
		}
	}

	match serde_json::to_string_pretty(&player) {
		Ok(json) => json,
		_ => return "Serialization failed.".to_string(),
	}
}
