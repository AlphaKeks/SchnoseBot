//! Fetch all global maps from the `GlobalAPI` and `KZ:GO` and put them into a unified data
//! structure.

use {
	crate::error::Result,
	chrono::NaiveDateTime,
	gokz_rs::schnose_api::{self, maps::Course},
	serde::{Deserialize, Serialize},
};

/// Custom version of [`gokz_rs::maps::Map`] with some additional fields for convenience.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalMap {
	pub id: u16,
	pub name: String,
	pub tier: u8,
	pub courses: Vec<Course>,
	pub kzt: bool,
	pub skz: bool,
	pub vnl: bool,
	pub mapper_names: Vec<String>,
	pub mapper_ids: Vec<u64>,
	pub validated: bool,
	pub filesize: u64,
	pub created_on: NaiveDateTime,
	pub updated_on: NaiveDateTime,
	pub url: String,
	pub thumbnail: String,
}

/// Gets called once at the start to fetch and process all maps.
pub async fn init(gokz_client: &gokz_rs::Client) -> Result<Vec<GlobalMap>> {
	Ok(schnose_api::get_global_maps(gokz_client)
		.await?
		.into_iter()
		.filter_map(|global_map| {
			let kzt = global_map.courses[0].kzt;
			let skz = global_map.courses[0].skz;
			let vnl = global_map.courses[0].vnl;
			let url = format!("https://kzgo.eu/maps/{}", &global_map.name);
			let thumbnail = format!(
				"https://raw.githubusercontent.com/KZGlobalTeam/map-images/master/images/{}.jpg",
				&global_map.name
			);
			let (created_on, _) = global_map.created_on.rsplit_once('.')?;
			let (updated_on, _) = global_map.updated_on.rsplit_once('.')?;
			Some(GlobalMap {
				id: global_map.id,
				name: global_map.name,
				tier: global_map.tier,
				courses: global_map.courses,
				kzt,
				skz,
				vnl,
				mapper_names: vec![global_map.mapper_name],
				mapper_ids: vec![global_map
					.mapper_steam_id64
					.parse()
					.ok()?],
				validated: global_map.validated,
				filesize: global_map.filesize.parse().ok()?,
				created_on: NaiveDateTime::parse_from_str(created_on, "%Y-%m-%d %H:%M:%S").ok()?,
				updated_on: NaiveDateTime::parse_from_str(updated_on, "%Y-%m-%d %H:%M:%S").ok()?,
				url,
				thumbnail,
			})
		})
		.collect())
}
