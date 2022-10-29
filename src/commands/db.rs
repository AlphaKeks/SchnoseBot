use crate::{event_handler::interaction_create::SchnoseResponseData, util::UserSchema};

use bson::doc;

use serenity::{
	builder::{CreateApplicationCommand, CreateEmbed},
	model::user::User,
};

pub fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	cmd.name("db").description("Check your current database entries.")
}

pub async fn run(collection: &mongodb::Collection<UserSchema>, user: &User) -> SchnoseResponseData {
	// try to access database
	match collection.find_one(doc! { "discordID": user.id.to_string() }, None).await {
		Err(why) => {
			log::error!(
				"[{}]: {} => {}\n{:#?}",
				file!(),
				line!(),
				"Failed to access database.",
				why
			);

			return SchnoseResponseData::Message(String::from("Failed to access database."));
		},
		Ok(document) => match document {
			None => {
				return SchnoseResponseData::Message(String::from(
					"You don't have any database entries yet.",
				));
			},
			Some(db_entry) => {
				let embed = CreateEmbed::default()
					.color((116, 128, 194))
					.title(format!("{}'s database entries", &user.name))
					.description(format!(
						"
						> discordID: {}
						> steamID: {}
						> mode: {}
						",
						db_entry.discordID,
						match db_entry.steamID {
							None => String::from("none"),
							Some(steam_id) => steam_id,
						},
						match db_entry.mode {
							None => String::from("none"),
							Some(mode) => mode,
						}
					))
					.to_owned();

				return SchnoseResponseData::Embed(embed);
			},
		},
	}
}
