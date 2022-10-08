use std::env;

use crate::SchnoseCommand;
use mongodb::options::{ClientOptions, ResolverConfig};
use serenity::model::application::interaction::{Interaction, InteractionResponseType};
use serenity::prelude::Context;

pub async fn interaction_create(ctx: Context, interaction: Interaction) {
	dotenv::dotenv().expect("Failed to load env file");

	let mongo_client = match env::var("MONGODB") {
		Ok(token) => {
			match ClientOptions::parse_with_resolver_config(&token, ResolverConfig::cloudflare())
				.await
			{
				Ok(options) => match mongodb::Client::with_options(options) {
					Ok(h) => h,
					Err(why) => panic!("{why}"),
				},
				Err(why) => panic!("{why}"),
			}
		}
		Err(why) => panic!("Failed to connect to database: {:#?}", why),
	};

	if let Interaction::ApplicationCommand(cmd) = interaction {
		println!("received interaction: {}", cmd.data.name);

		let data: SchnoseCommand = match cmd.data.name.as_str() {
			"ping" => crate::commands::ping::run(&cmd.data.options),
			"wr" => crate::commands::wr::run(&cmd.user, &cmd.data.options, &mongo_client).await,
			"db" => crate::commands::db::run(&cmd.user, &mongo_client).await,
			"setsteam" => {
				crate::commands::setsteam::run(&cmd.user, &cmd.data.options, &mongo_client).await
			}
			"mode" => crate::commands::mode::run(&cmd.user, &cmd.data.options, &mongo_client).await,
			_ => SchnoseCommand::Message(String::from("unknown command")),
		};

		if let Err(why) = cmd
			.create_interaction_response(&ctx.http, |res| {
				res.kind(InteractionResponseType::ChannelMessageWithSource)
					.interaction_response_data(|msg| match data {
						SchnoseCommand::Message(message) => msg.content(message),
						SchnoseCommand::Embed(embed) => msg.set_embed(embed),
					})
			})
			.await
		{
			println!("interaction failed: {:#?}", why);
		}
	}
}
