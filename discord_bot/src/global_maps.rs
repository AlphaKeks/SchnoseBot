//! Fetch all global maps from the `GlobalAPI` and `KZ:GO` and put them into a unified data
//! structure.

use {
	crate::error::Result,
	chrono::NaiveDateTime,
	gokz_rs::{
		schnose_api::{self, maps::Course},
		SteamID, Tier,
	},
	serde::{Deserialize, Serialize},
};

pub static BASED_MAPS: [&str; 27] = [
	"kz_epiphany_v2", "kz_lionharder", "kz_spacemario_h", "kz_mz", "kz_technical_difficulties",
	"kz_bhop_badges3", "kz_village", "kz_lionheart", "kz_oloramasa", "kz_haste", "kz_drops_od",
	"kz_otakuroom", "kzpro_wrath", "kz_custos", "kz_erratum_v2", "kz_shell", "kz_avoria",
	"kz_bhop_essence", "kz_okaychamp", "kz_ladderall", "kz_dale", "kz_2seasons_winter_final",
	"kz_gy_agitation", "kz_after_agitation_easy_fix", "kz_imaginary_final", "kz_adv_cursedjourney",
	"kz_exps_cursedjourney",
];

/// Custom version of [`gokz_rs::maps::Map`] with some additional fields for convenience.
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
pub async fn init(gokz_client: &gokz_rs::Client) -> Result<Vec<GlobalMap>> {
	Ok(schnose_api::get_global_maps(gokz_client)
		.await?
		.into_iter()
		.map(|global_map| {
			let kzt = global_map.courses[0].kzt;
			let skz = global_map.courses[0].skz;
			let vnl = global_map.courses[0].vnl;
			let url = format!("https://kzgo.eu/maps/{}", &global_map.name);
			let thumbnail = format!(
				"https://raw.githubusercontent.com/KZGlobalTeam/map-images/master/images/{}.jpg",
				&global_map.name
			);

			GlobalMap {
				id: global_map.id,
				name: global_map.name,
				tier: global_map.tier,
				courses: global_map.courses,
				kzt,
				skz,
				vnl,
				mapper_name: global_map.mapper_name,
				mapper_steam_id: global_map.mapper_steam_id,
				filesize: global_map.filesize,
				created_on: global_map.created_on,
				updated_on: global_map.updated_on,
				url,
				thumbnail,
			}
		})
		.collect())
}
