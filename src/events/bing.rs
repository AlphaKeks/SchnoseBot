use rand::Rng;
use serenity::{model::channel::Message, prelude::Context};

pub async fn message(ctx: Context, message: Message) {
	if message.content.to_lowercase().starts_with("bing?") {
		let chilling: u8 = rand::thread_rng().gen_range(0..=1);

		let chilling = if message.author.id == 241247299769073665 {
			"<:whatsapp:998940776136450128>"
		} else if message.author.id == 291585142164815873 // me
			|| message.author.id == 295966419261063168 // iBrahizy
			|| chilling == 1
		{
			"chilling 🥶"
		} else {
			"no <:joePensive:975446358796410890>"
		};

		if let Err(why) = message.reply(&ctx.http, chilling).await {
			log::error!("`bing?` failed: {:#?}", why);
		}
	}
}
