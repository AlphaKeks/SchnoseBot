use serenity::async_trait;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

use gokz::*;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
	async fn ready(&self, _: Context, ready: Ready) {
		println!("{} is now online!", ready.user.name);
	}
}

#[tokio::main]
async fn main() -> Result<(), APIError> {
	dotenv::dotenv()?;

	let token = dotenv::var("DISCORD_TOKEN")?;

	let mut client = Client::builder(token, GatewayIntents::empty())
		.event_handler(Handler)
		.await
		.expect("Error trying to create a client");

	if let Err(why) = client.start().await {
		println!("Client error: {:#?}", why);
	}

	Ok(())
}
