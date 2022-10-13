use crate::util::UserSchema;
use bson::doc;
use serenity::builder::{CreateApplicationCommand, CreateEmbed};
use serenity::model::user::User;

use crate::SchnoseCommand;

pub fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	cmd.name("db")
		.description("Check your current database entries")
}

pub async fn run(user: &User, mongo_client: &mongodb::Client) -> SchnoseCommand {
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
						match data.steamID {
							Some(steam_id) => steam_id.to_string(),
							None => String::from("none"),
						},
						if let Some(mode) = data.mode {
							mode
						} else {
							String::from("none")
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
