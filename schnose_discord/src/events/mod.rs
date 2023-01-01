mod message;
mod ready;

use {
	crate::GlobalState,
	poise::{serenity_prelude::Context, Event},
};

pub async fn handler(
	ctx: &Context,
	event: &Event<'_>,
	_framework: poise::FrameworkContext<'_, GlobalState, crate::SchnoseError>,
	global_state: &GlobalState,
) -> Result<(), crate::SchnoseError> {
	match event {
		Event::Message { new_message } => message::handle(ctx, global_state, new_message).await?,
		Event::Ready { data_about_bot } => ready::handle(ctx, global_state, data_about_bot).await?,
		_ => {},
	}

	Ok(())
}
