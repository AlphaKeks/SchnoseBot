extern crate wasm_bindgen;

use std::str::FromStr;

use chrono::NaiveDateTime;
use serde::Serialize;
use wasm_bindgen::prelude::*;

mod core;
mod structs;

use crate::core::*;

type JSResult = Result<String, String>;

#[wasm_bindgen]
pub async fn api_status() -> JSResult {
	let client = reqwest::Client::new();

	let result = match crate::core::api_status(&client).await {
		Ok(data) => data,
		Err(err) => return Err(err.to_string()),
	};

	match serde_json::to_string_pretty(&result) {
		Ok(json) => Ok(json),
		Err(_) => Err("GlobalAPI returned invalid data.".to_string()),
	}
}

#[wasm_bindgen]
pub async fn get_map(map_name: String) -> JSResult {
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

	let client = reqwest::Client::new();

	let map = match crate::core::get_map(NameOrId::Name(map_name), &client).await {
		Ok(data) => data,
		Err(err) => return Err(err.to_string()),
	};

	let kzgo_map = match crate::core::get_map_kzgo(map.name.clone(), &client).await {
		Ok(data) => data,
		Err(err) => return Err(err.to_string()),
	};

	let mut mappers = vec![];
	for i in 0..kzgo_map.mapper_names.len() {
		mappers.push(format!(
			"[{}](https://steamcommunity.com/profiles/{})",
			kzgo_map.mapper_names[i], kzgo_map.mapper_ids[i]
		));
	}

	let filters = match crate::core::get_filters(map.id, 0, &client).await {
		Ok(data) => data,
		Err(err) => return Err(err.to_string()),
	};

	let date = match NaiveDateTime::from_str(&map.created_on) {
		Ok(str) => str,
		_ => return Err("Error parsing date.".to_string()),
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
		date: format!("<t:{}:d>", date.timestamp() / 1000).to_string(),
		filters: embed_filters,
	};

	match serde_json::to_string_pretty(&result) {
		Ok(str) => Ok(str),
		Err(err) => return Err(err.to_string()),
	}
}
