use {
	super::choices::TierChoice,
	crate::{
		error::{Error, Result},
		global_maps::GlobalMap,
		Context, State,
	},
	log::trace,
	rand::Rng,
};

/// Get a random map name from the global map pool.
///
/// This command will simply select a random map from the global map pool. You may specify a \
/// `tier` if you want to.
#[poise::command(slash_command, on_error = "Error::handle_command")]
pub async fn random(
	ctx: Context<'_>,
	#[description = "Filter by map difficulty."] tier: Option<TierChoice>,
) -> Result<()> {
	trace!("[/random ({})]", ctx.author().tag());
	trace!("> `tier`: {tier:?}");
	ctx.defer().await?;

	let global_maps = ctx
		.global_maps()
		.iter()
		.filter(|map| tier.map_or(true, |tier| map.tier as u8 == tier as u8))
		.collect::<Vec<_>>();

	let rng = rand::thread_rng().gen_range(0..global_maps.len());

	let GlobalMap { name, tier, .. } = &global_maps[rng];

	ctx.say(format!("🎲 {name} ({tier})"))
		.await?;

	Ok(())
}
