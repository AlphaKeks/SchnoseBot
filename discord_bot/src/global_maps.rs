//! Fetch all global maps from the `GlobalAPI` and `KZ:GO` and put them into a unified data
//! structure.

use {
	crate::error::Result,
	chrono::NaiveDateTime,
	gokz_rs::{GlobalAPI, KZGO},
	serde::{Deserialize, Serialize},
};

/// Custom version of [`gokz_rs::maps::Map`] which has additional info from `KZ:GO` and proper data
/// types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalMap {
	pub id: u16,
	pub name: String,
	pub tier: u8,
	pub courses: u8,
	pub sp: bool,
	pub vp: bool,
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
	let global_maps = GlobalAPI::get_maps(true, Some(9999), gokz_client).await?;
	let mut kzgo_maps = KZGO::get_maps(gokz_client)
		.await?
		.into_iter();

	Ok(global_maps
		.into_iter()
		.filter_map(|global_map| {
			let kzgo_map = kzgo_maps.find_map(|map| {
				if let Some(map_name) = &map.name {
					if map_name == &global_map.name {
						return Some(map);
					}
				}
				None
			})?;

			Some(GlobalMap {
				id: global_map.id as u16,
				name: global_map.name,
				tier: global_map.difficulty as u8,
				courses: kzgo_map.bonuses?,
				sp: kzgo_map.sp?,
				vp: kzgo_map.vp?,
				mapper_names: kzgo_map
					.mapperNames
					.into_iter()
					.flatten()
					.collect(),
				mapper_ids: kzgo_map
					.mapperIds
					.into_iter()
					.filter_map(|id| id?.parse::<u64>().ok())
					.collect(),
				validated: global_map.validated,
				filesize: global_map.filesize as u64,
				created_on: NaiveDateTime::parse_from_str(
					&global_map.created_on, "%Y-%m-%dT%H:%M:%S",
				)
				.ok()?,
				updated_on: NaiveDateTime::parse_from_str(
					&global_map.updated_on, "%Y-%m-%dT%H:%M:%S",
				)
				.ok()?,
                url: format!("https://kzgo.eu/maps/{}", &kzgo_map.name.as_ref()?),
				thumbnail: format!("https://raw.githubusercontent.com/KZGlobalTeam/map-images/master/images/{}.jpg", kzgo_map.name?)
			})
		})
		.collect())
}
