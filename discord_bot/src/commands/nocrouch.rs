use {
	crate::{error::Error, Context},
	log::trace,
};

/// Approximate a nocrouch jump's potential distance.
///
/// This is by no means 100% accurate and also not very hard to calculate yourself. The command \
/// only really exists for the sake of convenience.
///
/// ```
/// potential_distance = actual_distance + (max_speed / 128) * 4
/// ```
#[poise::command(slash_command, on_error = "Error::handle_command")]
pub async fn nocrouch(
	ctx: Context<'_>,
	#[description = "The distance of your jump"] distance: f64,
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
