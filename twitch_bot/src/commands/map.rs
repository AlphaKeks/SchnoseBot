use {
	crate::Result,
	schnosebot::global_maps::GlobalMap,
	tokio::time::{sleep, Duration},
};

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
	let plural = if bonuses == 1 { "" } else { "es" };

	sleep(Duration::from_millis(727)).await;

	Ok(format!(
		"{name} (T{tier}) - {bonuses} Bonus{plural} - Made by {mapper_name} - Last Updated on {updated_on}"
	))
}
