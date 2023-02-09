use {
	super::choices::TierChoice,
	crate::{error::Error, global_maps::GlobalMap, Context, State},
	log::trace,
	rand::Rng,
};

#[poise::command(slash_command, on_error = "Error::handle_command")]
pub async fn random(
	ctx: Context<'_>, #[description = "Filter by map difficulty."] tier: Option<TierChoice>,
) -> Result<(), Error> {
	ctx.defer().await?;
	trace!("[/random] tier: `{tier:?}`");

	let global_maps = ctx
		.global_maps()
		.iter()
		.filter(|map| tier.map_or(true, |tier| map.tier == tier as u8))
		.collect::<Vec<_>>();

	let rng = rand::thread_rng().gen_range(0..global_maps.len());

	let GlobalMap { name, tier, .. } = &global_maps[rng];

	ctx.say(format!("🎲 {name} (T{tier})"))
		.await?;

	Ok(())
}
