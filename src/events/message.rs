use {
	log::info,
	serenity::{prelude::Context, model::prelude::Message},
	rand::{thread_rng, Rng},
};

pub(crate) async fn handle(ctx: &Context, msg: &Message) -> anyhow::Result<()> {
	if msg.content.to_lowercase().starts_with("bing?") {
		return chilling(ctx, msg).await;
	}

	return Ok(());
}

async fn chilling(ctx: &Context, msg: &Message) -> anyhow::Result<()> {
	let response = match msg.author.id.as_u64() {
		// AlphaKeks
		&291585142164815873 => "chilling 🥶",
		// jucci
		&241247299769073665 => "<:whatsapp:998940776136450128>",
		// everybody else
		_ => {
			if thread_rng().gen_bool(0.69 /* (͡ ͡° ͜ つ ͡͡°) */) {
				"chilling 🥶"
			} else {
				"no <:joePensive:975446358796410890>"
			}
		},
	};

	msg.reply(&ctx.http, response).await?;

	info!("{}: {}", &msg.author.name, &msg.content);
	info!("schnose: {}", response);

	return Ok(());
}
