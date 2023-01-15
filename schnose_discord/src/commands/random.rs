use {
	super::{handle_err, TierChoice, GLOBAL_MAPS},
	crate::SchnoseError,
	log::trace,
	rand::Rng,
};

/// Generate a random KZ map.
#[poise::command(slash_command, on_error = "handle_err")]
pub async fn random(
	ctx: crate::Context<'_>,
	#[description = "Filter by map difficulty."] tier: Option<TierChoice>,
) -> Result<(), SchnoseError> {
	ctx.defer().await?;

	trace!("[/random] tier: `{:?}`", &tier);

	let maps = (*GLOBAL_MAPS)
		.iter()
		.filter(|map| match tier {
			Some(tier) => map.difficulty == tier as u8,
			None => true,
		})
		.collect::<Vec<_>>();

	let rng = rand::thread_rng().gen_range(0..maps.len());

	let map = &maps[rng];

	ctx.say(format!("🎲 `{} (T{})`", map.name, map.difficulty))
		.await?;

	Ok(())
}
