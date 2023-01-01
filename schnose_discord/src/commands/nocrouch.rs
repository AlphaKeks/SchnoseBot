use {super::handle_err, crate::SchnoseError};

/// Approximate the distance of a nocrouch jump.
#[poise::command(slash_command, on_error = "handle_err")]
pub async fn nocrouch(
	ctx: crate::Context<'_>,
	#[description = "Specify the distance of your nocrouch jump."] distance: f32,
	#[description = "Specify the max speed of your nocrouch jump."] max: f32,
) -> Result<(), SchnoseError> {
	let approx = distance + (max / 128.) * 4.;
	ctx.say(format!("Approximated distance: `{0:.4}`", approx)).await?;
	Ok(())
}
