mod commands;

use serenity::model::prelude::GuildId;

use std::collections::HashSet;
use std::error::Error;
use std::sync::Arc;

use serenity::async_trait;
use serenity::client::bridge::gateway::ShardManager;
use serenity::framework::StandardFramework;
use serenity::http::Http;
use serenity::model::application::command::Command;
use serenity::model::application::interaction::{Interaction, InteractionResponseType};
use serenity::model::gateway::Ready;
use serenity::prelude::*;

pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
	type Value = Arc<Mutex<ShardManager>>;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
	async fn ready(&self, ctx: Context, ready: Ready) {
		let data_collection = [commands::ping::data, commands::map::data];
		if let Ok(var) = dotenv::var("MODE") {
			match var.as_str() {
				"DEV" => {
					let guild_id = GuildId(
						dotenv::var("GUILD_ID")
							.expect("Expected GUILD_ID in environment")
							.parse()
							.expect("GUILD_ID must be an integer"),
					);

					for data in data_collection {
						let _ = GuildId::create_application_command(&guild_id, &ctx.http, |cmd| {
							data(cmd)
						})
						.await;
					}
				}
				"PROD" => {
					for data in data_collection {
						let _ =
							Command::create_global_application_command(&ctx.http, |cmd| data(cmd))
								.await;
					}
				}
				_ => (),
			}
		};

		println!("Connected as {}", ready.user.name);
	}

	#[allow(unused_variables)]
	async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
		if let Interaction::ApplicationCommand(command) = interaction {
			let func_collection = [commands::ping::execute, commands::map::execute];
			for action in func_collection {
				let _ = command
					.create_interaction_response(&ctx.http, |reply| {
						reply
							.kind(InteractionResponseType::ChannelMessageWithSource)
							.interaction_response_data(|data| {
								let h = action();
								data.content(h)
							})
					})
					.await;
			}
		};
	}
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
	dotenv::dotenv()?;
	let token = dotenv::var("DISCORD_TOKEN")?;

	let http = Http::new(&token);

	let (owners, _bot_id) = match http.get_current_application_info().await {
		Ok(info) => {
			let mut owners = HashSet::new();
			owners.insert(info.owner.id);

			(owners, info.id)
		}
		Err(why) => panic!("Could not access application info: {}", why),
	};

	let framework = StandardFramework::new().configure(|conf| conf.owners(owners));

	let intents = GatewayIntents::GUILDS
		| GatewayIntents::GUILD_MEMBERS
		| GatewayIntents::GUILD_MESSAGES
		| GatewayIntents::GUILD_MESSAGE_REACTIONS
		| GatewayIntents::MESSAGE_CONTENT;

	let mut client = Client::builder(&token, intents)
		.framework(framework)
		.event_handler(Handler)
		.await
		.expect("Error trying to create client.");

	{
		let mut data = client.data.write().await;
		data.insert::<ShardManagerContainer>(client.shard_manager.clone());
	}

	let shard_manager = client.shard_manager.clone();

	tokio::spawn(async move {
		tokio::signal::ctrl_c()
			.await
			.expect("Could not register ctrl+c handler.");
		shard_manager.lock().await.shutdown_all().await;
	});

	if let Err(why) = client.start().await {
		return Err(why.into());
	}

	Ok(())
}

// match glob("./src/commands/*.rs") {
/*
	Ok(paths) => {
		for file in paths {
			match file {
				Ok(data) => {
					let str = data.display().to_string();
					if !str.contains("mod.rs") {
						match str.strip_prefix("src/commands/") {
							_ => match str.strip_suffix(".rs") {
								Some(s) => D,
								None => (),
							},
						};
					}
				}
				_ => (),
			}
		}
	}
	_ => (),
};
*/
