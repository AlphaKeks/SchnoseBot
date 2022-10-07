use bson::doc;
use gokz_rs::global_api::GOKZModeName;
use serde::{Deserialize, Serialize};
use serenity::builder::{CreateApplicationCommand, CreateEmbed};
use serenity::model::user::User;

use crate::SchnoseCommand;

pub fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	cmd.name("db")
		.description("Check your current database entries")
}

pub async fn run(user: &User, mongo_client: &mongodb::Client) -> SchnoseCommand {
	#[derive(Debug, Serialize, Deserialize)]
	#[allow(dead_code, non_snake_case)]
	struct UserSchema {
		name: String,
		discordID: String,
		steamID: Option<String>,
		mode: Option<GOKZModeName>,
	}
	let database = mongo_client
		.database("gokz")
		.collection::<UserSchema>("users");

	match database
		.find_one(
			doc! {
				"discordID": user.id.to_string()
			},
			None,
		)
		.await
	{
		Ok(doc) => match doc {
			Some(data) => {
				let embed = CreateEmbed::default()
					.title(format!("{}'s database entries:", data.name))
					.description(format!(
						"
						discordID: {}\n
						steamID: {}\n
						mode: {}
						",
						data.discordID,
						data.steamID.unwrap_or(String::from("none")),
						if let Some(mode) = data.mode {
							mode.as_str()
						} else {
							"none"
						}
					))
					.to_owned();

				SchnoseCommand::Embed(embed)
			}
			None => SchnoseCommand::Message(String::from("my balls hurt.")),
		},
		Err(_) => SchnoseCommand::Message(String::from("You don't have any database entries.")),
	}
}
