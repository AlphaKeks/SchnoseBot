use rand::Rng;
use serenity::{model::prelude::Message, prelude::Context};

pub async fn handle(ctx: Context, msg: Message) {
	if msg.content.to_lowercase().starts_with("bing?") {
		let chilling = if msg.author.id == 291585142164815873 {
			69
		} else if msg.author.id == 295966419261063168 {
			0
		} else {
			rand::thread_rng().gen_range(0..100)
		};

		let chilling =
			if chilling > 50 { "chilling 🥶" } else { "no <:joePensive:975446358796410890>" };

		if let Err(why) = msg.reply(&ctx.http, chilling).await {
			log::error!("`bing?` failed: {:#?}", why);
		}
	}
}
