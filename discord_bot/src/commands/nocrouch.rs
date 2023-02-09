use {
	crate::{error::Error, Context},
	log::trace,
};

/// Approximate a nocrouch jump's potential distance. (NOT PERFECTLY ACCURATE)
#[poise::command(slash_command, on_error = "Error::handle_command")]
pub async fn nocrouch(
	ctx: Context<'_>, #[description = "The distance of your jump"] distance: f64,
	#[description = "The max speed of your jump"] max: f64,
) -> Result<(), Error> {
	trace!("[/nocrouch ({})]", ctx.author().tag());
	trace!("> `distance`: {distance:?}");
	trace!("> `max`: {max:?}");

	let potential_distance = (max / 128f64).mul_add(4f64, distance);
	ctx.say(format!("Approximated distance: `{potential_distance:.4}`"))
		.await?;
	Ok(())
}
