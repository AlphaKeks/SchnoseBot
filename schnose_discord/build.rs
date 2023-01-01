use std::{env, fs, path::Path};

fn main() {
	let out_dir = env::var_os("OUT_DIR").unwrap();
	let dest_path = Path::new(&out_dir).join("global_maps.json");
	let client = reqwest::blocking::Client::new();

	let response = client
		.get("https://kztimerglobal.com/api/v2/maps?is_validated=true&limit=9999")
		.send()
		.unwrap();

	let maps = response.json::<Vec<Map>>().unwrap();

	fs::write(dest_path, serde_json::to_string(&maps).unwrap()).unwrap()
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Map {
	pub id: u32,
	pub name: String,
	pub filesize: u64,
	pub validated: bool,
	pub difficulty: u8,
	pub created_on: String,
	pub updated_on: String,
	pub approved_by_steamid64: String,
	pub workshop_url: String,
	pub download_url: Option<String>,
}
