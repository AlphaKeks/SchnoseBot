use std::env;

use crate::SchnoseCommand;
use mongodb::options::{ClientOptions, ResolverConfig};
use serenity::model::application::interaction::Interaction;
use serenity::model::prelude::interaction::InteractionResponseType;
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
		match cmd.data.name.as_str() {
			"db" => (),
			_ => {
				if let Err(_) = cmd.defer(&ctx.http).await {
					return;
				}
			}
		}

		println!("received interaction: {}", cmd.data.name);

		let data: SchnoseCommand = match cmd.data.name.as_str() {
			"ping" => crate::commands::ping::run(&cmd.data.options),
			"db" => crate::commands::db::run(&cmd.user, &mongo_client).await,
			"setsteam" => {
				crate::commands::setsteam::run(&cmd.user, &cmd.data.options, &mongo_client).await
			}
			"mode" => crate::commands::mode::run(&cmd.user, &cmd.data.options, &mongo_client).await,
			"wr" => crate::commands::wr::run(&cmd.user, &cmd.data.options, &mongo_client).await,
			"bwr" => crate::commands::bwr::run(&cmd.user, &cmd.data.options, &mongo_client).await,
			"pb" => crate::commands::pb::run(&cmd.user, &cmd.data.options, &mongo_client).await,
			"bpb" => crate::commands::bpb::run(&cmd.user, &cmd.data.options, &mongo_client).await,
			"recent" => {
				crate::commands::recent::run(&cmd.user, &cmd.data.options, &mongo_client).await
			}
			_ => SchnoseCommand::Message(String::from("unknown command")),
		};

		match cmd.data.name.as_str() {
			"db" => {
				if let Err(why) = cmd
					.create_interaction_response(&ctx.http, |h| {
						h.kind(InteractionResponseType::ChannelMessageWithSource)
							.interaction_response_data(|msg| match data {
								SchnoseCommand::Embed(embed) => {
									msg.ephemeral(true).set_embed(embed)
								}
								_ => unreachable!("This should always return an embed."),
							})
					})
					.await
				{
					println!("interaction failed: {:#?}", why);
				}
			}
			_ => {
				if let Err(why) = cmd
					.edit_original_interaction_response(&ctx.http, |res| {
						match cmd.data.name.as_str() {
							_ => match data {
								SchnoseCommand::Message(message) => res.content(message),
								SchnoseCommand::Embed(embed) => res.set_embed(embed),
							},
						}
					})
					.await
				{
					println!("interaction failed: {:#?}", why);
				}
			}
		}
	}
}
