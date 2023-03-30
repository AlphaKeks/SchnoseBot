use crate::{global_maps::GlobalMap, Result};

#[tracing::instrument]
pub async fn execute(map: GlobalMap) -> Result<String> {
	let GlobalMap {
		id: _,
		name,
		tier,
		courses,
		kzt: _,
		skz: _,
		vnl: _,
		mapper_name,
		mapper_steam_id: _,
		filesize: _,
		created_on: _,
		updated_on,
		url: _,
		thumbnail: _,
	} = map;

	let tier = tier as u8;
	let bonuses = courses.len() - 1;

	Ok(format!(
		"{name} (T{tier}) - {bonuses} Bonuses - Made by {mapper_name} - Last Updated on {updated_on}"
	))
}
