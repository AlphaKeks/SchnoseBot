use rand::Rng;
use serenity::{model::prelude::Message, prelude::Context};

pub async fn handle(ctx: Context, msg: Message) {
	if msg.content.to_lowercase().starts_with("bing?") {
		let chilling = if msg.author.id == 291585142164815873 {
			69
		} else {
			rand::thread_rng().gen_range(0..100)
		};

		// whatsapp for coronel whatsapp
		let chilling = if msg.author.id == 241247299769073665 {
			"<:whatsapp:998940776136450128>"
		} else if chilling > 50 {
			"chilling 🥶"
		} else {
			"no <:joePensive:975446358796410890>"
		};

		if let Err(why) = msg.reply(&ctx.http, chilling).await {
			log::error!("`bing?` failed: {:#?}", why);
		}
	}
}
