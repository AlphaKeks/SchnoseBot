use {
	crate::GlobalState,
	poise::serenity_prelude::{Context, Message},
	rand::{thread_rng, Rng},
};

pub async fn handle(
	ctx: &Context, _global_state: &GlobalState, message: &Message,
) -> Result<(), crate::SchnoseError> {
	if message
		.content
		.to_lowercase()
		.starts_with("bing?")
	{
		let response = match message.author.id.as_u64() {
			// AlphaKeks
			291585142164815873 => "chilling 🥶",
			// jucci
			241247299769073665 => "<:whatsapp:998940776136450128>",
			// everybody else
			_ => {
				if thread_rng().gen_bool(0.69 /* (͡ ͡° ͜ つ ͡͡°) */) {
					"chilling 🥶"
				} else {
					"no <:joePensive:975446358796410890>"
				}
			}
		};

		message.reply(ctx, response).await?;
	}

	Ok(())
}
