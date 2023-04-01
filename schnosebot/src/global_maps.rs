use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};

use {
	chrono::NaiveDateTime,
	color_eyre::Result,
	gokz_rs::{
		global_api,
		schnose_api::{self, maps::Course},
		MapIdentifier, Mode, SteamID, Tier,
	},
	serde::{Deserialize, Serialize},
};

/// Custom version of [`global_api::Map`] with some additional fields for convenience.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalMap {
	pub id: u16,
	pub name: String,
	pub tier: Tier,
	pub courses: Vec<Course>,
	pub kzt: bool,
	pub skz: bool,
	pub vnl: bool,
	pub mapper_name: String,
	pub mapper_steam_id: Option<SteamID>,
	pub filesize: u64,
	pub created_on: NaiveDateTime,
	pub updated_on: NaiveDateTime,
	pub url: String,
	pub thumbnail: String,
}

/// Gets called once at the start to fetch and process all maps.
#[tracing::instrument]
pub async fn init(gokz_client: &gokz_rs::Client, global_only: bool) -> Result<Vec<GlobalMap>> {
	let mut maps = Vec::new();
	let filters = global_api::record_filters::get_filters(
		global_api::record_filters::index::Params {
			tickrates: Some(128),
			stages: Some(0),
			limit: Some(9999),
			..Default::default()
		},
		gokz_client,
	)
	.await?;

	for map in if global_only {
		schnose_api::get_global_maps(gokz_client).await?
	} else {
		schnose_api::get_maps(gokz_client).await?
	} {
		let url = format!("https://kzgo.eu/maps/{}", &map.name);
		let thumbnail = format!(
			"https://raw.githubusercontent.com/KZGlobalTeam/map-images/master/images/{}.jpg",
			&map.name
		);

		maps.push(GlobalMap {
			id: map.id,
			name: map.name,
			tier: map.tier,
			courses: map.courses,
			kzt: filters
				.iter()
				.any(|filter| filter.map_id == map.id && filter.mode == Mode::KZTimer),
			skz: filters
				.iter()
				.any(|filter| filter.map_id == map.id && filter.mode == Mode::SimpleKZ),
			vnl: filters
				.iter()
				.any(|filter| filter.map_id == map.id && filter.mode == Mode::Vanilla),
			mapper_name: map.mapper_name,
			mapper_steam_id: map.mapper_steam_id,
			filesize: map.filesize,
			created_on: map.created_on,
			updated_on: map.updated_on,
			url,
			thumbnail,
		});
	}

	maps.sort_unstable_by(|a, b| a.name.cmp(&b.name));

	Ok(maps)
}

const MIN_SCORE: i64 = 1;

pub fn fuzzy_find_map(
	map_identifier: impl Into<MapIdentifier>,
	map_pool: &[GlobalMap],
) -> Option<GlobalMap> {
	let map_identifier = map_identifier.into();
	match map_identifier {
		MapIdentifier::ID(map_id) => {
			map_pool
				.iter()
				.find_map(|map| if map.id == map_id { Some(map.to_owned()) } else { None })
		}
		MapIdentifier::Name(map_name) => {
			let fzf = SkimMatcherV2::default();
			let map_name = map_name.to_lowercase();
			map_pool
				.iter()
				.filter_map(|map| {
					let score = fzf.fuzzy_match(&map.name, &map_name)?;
					if score >= MIN_SCORE || map_name.is_empty() {
						return Some((score, map.to_owned()));
					}
					None
				})
				.max_by(|(a_score, _), (b_score, _)| a_score.cmp(b_score))
				.map(|(_, map)| map)
		}
	}
}

pub fn fuzzy_find_map_name(
	map_identifier: impl Into<MapIdentifier>,
	map_pool: &[GlobalMap],
) -> Option<String> {
	fuzzy_find_map(map_identifier, map_pool).map(|map| map.name)
}
