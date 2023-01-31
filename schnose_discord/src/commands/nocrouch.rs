use {super::handle_err, crate::SchnoseError};

/// Approximate the distance of a nocrouch jump.
#[poise::command(slash_command, on_error = "handle_err")]
pub async fn nocrouch(
	ctx: crate::Context<'_>,
	#[description = "Specify the distance of your nocrouch jump."] distance: f32,
	#[description = "Specify the max speed of your nocrouch jump."] max: f32,
) -> Result<(), SchnoseError> {
	let approx = (max / 128.0).mul_add(4.0, distance);
	ctx.say(format!("Approximated distance: `{approx:.4}`"))
		.await?;
	Ok(())
}
