use crate::{
	error::{Error, Result},
	Context,
};

/// Approximate a nocrouch jump's potential distance.
///
/// This command will calculate a potential distance for a jump from a `max` speed and the actual \
/// `distance` that you landed. This is a very optimistic approximation and looks like this:
///
/// ```
/// potential_distance = actual_distance + (max_speed / 128) * 4
/// ```
///
/// If you don't crouch at the end of your jump you are missing out on _4_ ticks of airtime. So we \
/// take your `max` speed and dividie it by the tickrate to get the distance for each tick. We \
/// then multiply by 4 to make up for the 4 lost ticks of airtime. All of this assumes that you \
/// didn't have any loss on your last strafe and went perfectly straight (perfect airpath).
#[tracing::instrument(skip(ctx), fields(user = ctx.author().tag()))]
#[poise::command(slash_command, on_error = "Error::handle_command")]
pub async fn nocrouch(
	ctx: Context<'_>,
	#[description = "The distance of your jump"] distance: f64,
	#[description = "The max speed of your jump"] max: f64,
) -> Result<()> {
	let potential_distance = (max / 128f64).mul_add(4f64, distance);
	ctx.say(format!("Approximated distance: `{potential_distance:.4}`"))
		.await?;
	Ok(())
}
