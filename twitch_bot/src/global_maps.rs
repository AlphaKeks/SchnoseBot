use {
	crate::Result,
	chrono::NaiveDateTime,
	gokz_rs::{
		global_api,
		schnose_api::{self, maps::Course},
		Mode, SteamID, Tier,
	},
	serde::{Deserialize, Serialize},
	tracing::info,
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
pub async fn init(gokz_client: &gokz_rs::Client) -> Result<Vec<GlobalMap>> {
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

	for global_map in schnose_api::get_global_maps(gokz_client).await? {
		let url = format!("https://kzgo.eu/maps/{}", &global_map.name);
		let thumbnail = format!(
			"https://raw.githubusercontent.com/KZGlobalTeam/map-images/master/images/{}.jpg",
			&global_map.name
		);

		maps.push(GlobalMap {
			id: global_map.id,
			name: global_map.name,
			tier: global_map.tier,
			courses: global_map.courses,
			kzt: filters
				.iter()
				.any(|filter| filter.map_id == global_map.id && filter.mode == Mode::KZTimer),
			skz: filters
				.iter()
				.any(|filter| filter.map_id == global_map.id && filter.mode == Mode::SimpleKZ),
			vnl: filters
				.iter()
				.any(|filter| filter.map_id == global_map.id && filter.mode == Mode::Vanilla),
			mapper_name: global_map.mapper_name,
			mapper_steam_id: global_map.mapper_steam_id,
			filesize: global_map.filesize,
			created_on: global_map.created_on,
			updated_on: global_map.updated_on,
			url,
			thumbnail,
		});
	}

	maps.sort_unstable_by(|a, b| a.name.cmp(&b.name));

	info!("Fetched {} global maps.", maps.len());

	Ok(maps)
}
