mod commands;

use std::env;

use serenity::builder::CreateEmbed;
use serenity::framework::standard::macros::group;
use serenity::framework::StandardFramework;
use serenity::model::application::command::Command;
use serenity::model::application::interaction::{Interaction, InteractionResponseType};
use serenity::model::gateway::Ready;
use serenity::model::prelude::GuildId;
use serenity::prelude::GatewayIntents;
use serenity::Client;
use serenity::{
	async_trait,
	prelude::{Context, EventHandler},
};

pub enum SchnoseCommand {
	Message(String),
	Embed(CreateEmbed),
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
	async fn ready(&self, ctx: Context, ready: Ready) {
		println!("Connected as {}.", ready.user.tag());

		let dev_guild = GuildId(
			env::var("DEV_GUILD")
				.expect("no `DEV_GUILD` environment variable found")
				.parse()
				.expect("`DEV_GUILD` must be an integer"),
		);

		let mode = env::var("MODE").expect("no `MODE` environment variable found");

		match mode.as_str() {
			"DEV" => {
				let commands =
					GuildId::set_application_commands(&dev_guild, &ctx.http, |commands| {
						// commands.create_application_command(|cmd| commands::ping::register(cmd))
						commands.create_application_command(|cmd| commands::map::register(cmd))
					})
					.await;

				let mut names = vec![];
				if let Ok(commands) = commands {
					for command in commands {
						names.push(command.name);
					}
				}

				println!("[{}] registered commands: {:#?}", mode, names);
			}
			"PROD" => {
				let commands = Command::create_global_application_command(&ctx.http, |cmd| {
					commands::ping::register(cmd)
				})
				.await;

				println!(
					"[{}] registered commands: {:#?}",
					mode,
					commands.map(|cmd| cmd.name)
				);
			}
			_ => panic!("invalid mode!"),
		}
	}

	async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
		if let Interaction::ApplicationCommand(cmd) = interaction {
			println!("received interaction: {}", cmd.data.name);

			let data: SchnoseCommand = match cmd.data.name.as_str() {
				"ping" => commands::ping::run(&cmd.data.options),
				"map" => commands::map::run(&cmd.data.options).await,
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
}

#[group]
struct General;

#[tokio::main]
async fn main() {
	dotenv::dotenv().expect("Failed to load env file");

	let token = env::var("DISCORD_TOKEN").expect("no discord token found");

	let framework = StandardFramework::new();

	let intents = GatewayIntents::GUILD_MESSAGES
		| GatewayIntents::DIRECT_MESSAGES
		| GatewayIntents::MESSAGE_CONTENT;

	let mut client = Client::builder(&token, intents)
		.framework(framework)
		.event_handler(Handler)
		.await
		.expect("fuck you thats why");

	if let Err(why) = client.start().await {
		panic!("client crashed: {:#?}", why)
	}
}
