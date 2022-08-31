use serde::{Deserialize, Serialize};

/* GlobalAPI */
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Ban {
	pub id: Option<u32>,
	pub ban_type: String,
	pub expires_on: String,
	pub steamid64: String,
	pub player_name: Option<String>,
	pub steam_id: Option<String>,
	pub notes: Option<String>,
	pub stats: Option<String>,
	pub server_id: u32,
	pub updated_by_id: String,
	pub created_on: String,
	pub updated_on: String,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Jumpstat {
	pub id: Option<u32>,
	pub server_id: u32,
	pub steamid64: String,
	pub player_name: Option<String>,
	pub steam_id: Option<String>,
	pub jump_type: u8,
	pub distance: f32,
	pub tickrate: u32,
	pub msl_count: u32,
	pub strafe_count: u32,
	pub is_crouch_bind: u32,
	pub is_forward_bind: u32,
	pub is_crouch_boost: u32,
	pub updated_by_id: u32,
	pub created_on: String,
	pub updated_on: String,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Map {
	pub id: u16,
	pub name: String,
	pub filesize: u128,
	pub validated: bool,
	pub difficulty: u8,
	pub created_on: String,
	pub updated_on: String,
	pub approved_by_steamid64: String,
	pub workshop_url: String,
	pub download_url: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Mode {
	pub id: u32,
	pub name: Option<String>,
	pub description: Option<String>,
	pub latest_version: u32,
	pub latest_version_description: Option<String>,
	pub website: Option<String>,
	pub repo: Option<String>,
	pub contact_steamid64: String,
	pub supported_tickrates: Option<Vec<u32>>,
	pub created_on: String,
	pub updated_on: String,
	pub updated_by_id: String,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Player {
	pub steamid64: String,
	pub steam_id: Option<String>,
	pub is_banned: bool,
	pub total_records: u32,
	pub name: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct RecordFilter {
	pub id: u32,
	pub map_id: u32,
	pub stage: u32,
	pub mode_id: u32,
	pub tickrate: u32,
	pub has_teleports: bool,
	pub created_on: String,
	pub updated_on: String,
	pub updated_by_id: String,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DisplayFilter {
	pub mode: String,
	pub display_mode: String,
	pub short_mode: String,
	pub mode_id: u16,
	pub icon: String,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct DisplayFilterCollection {
	pub kzt: DisplayFilter,
	pub skz: DisplayFilter,
	pub vnl: DisplayFilter,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct RecordFilterDistribution {
	record_filter_id: u32,
	pub c: f32,
	pub d: f32,
	pub loc: f32,
	pub scale: f32,
	pub top_scale: f32,
	pub created_on: String,
	pub updated_on: String,
	pub updated_by_id: String,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Record {
	pub id: u32,
	pub steamid64: String,
	pub player_name: String,
	pub steam_id: String,
	pub server_id: u32,
	pub map_id: u32,
	pub stage: u32,
	pub mode: String,
	pub tickrate: u32,
	pub time: f32,
	pub teleports: u32,
	pub created_on: String,
	pub updated_on: String,
	pub updated_by: u64,
	pub record_filter_id: u32,
	pub server_name: String,
	pub map_name: String,
	pub points: u32,
	pub replay_id: u32,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Place {
	pub steamid64: String,
	pub steam_id: String,
	pub count: u32,
	pub player_name: String,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Replay {
	pub id: u32,
	pub steamid64: String,
	pub server_id: u32,
	pub record_filter_id: u32,
	pub time: f32,
	pub teleports: u32,
	pub created_on: String,
	pub updated_on: String,
	pub updated_by: u64,
	pub points: u32,
	pub replay_id: u32,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Server {
	pub id: u32,
	pub port: u32,
	pub ip: String,
	pub name: String,
	pub owner_steamid64: String,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct APIStatusShort {
	pub status: String,
	pub frontend: String,
	pub backend: String,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct APIStatusPage {
	pub id: String,
	pub name: String,
	pub url: String,
	pub time_zone: String,
	pub updated_at: String,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct APIStatusComponent {
	pub id: String,
	pub name: String,
	pub status: String,
	pub created_at: String,
	pub updated_at: String,
	pub position: i32,
	pub description: String,
	pub showcase: bool,
	pub start_date: Option<String>,
	pub group_id: Option<String>,
	pub group: bool,
	pub only_show_if_degraded: bool,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct APIStatusStatus {
	pub indicator: String,
	pub description: String,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct APIStatus {
	pub page: APIStatusPage,
	pub components: Vec<APIStatusComponent>,
	pub incidents: Vec<Option<String>>,
	pub scheduled_maintenances: Vec<Option<String>>,
	pub status: APIStatusStatus,
}

/* KZ:GO API */
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct KZGOMap {
	pub _id: String,
	pub name: String,
	pub id: u16,
	pub tier: u8,
	#[serde(rename = "workshopId")]
	pub workshop_id: String,
	pub bonuses: u8,
	pub sp: bool,
	pub vp: bool,
	#[serde(rename = "mapperNames")]
	pub mapper_names: Vec<String>,
	#[serde(rename = "mapperIds")]
	pub mapper_ids: Vec<String>,
	pub date: String,
}
