use crate::{
	event_handler::interaction_create::{Metadata, SchnoseResponseData},
	util::UserSchema,
};

use bson::doc;

use serenity::builder::{CreateApplicationCommand, CreateEmbed};

pub fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	cmd.name("db").description("Check your current database entries.")
}

pub async fn run(metadata: Metadata, collection: &mongodb::Collection<UserSchema>) {
	// try to access database
	match collection
		.find_one(doc! { "discordID": metadata.cmd.user.id.to_string() }, None)
		.await
	{
		Err(why) => {
			log::error!(
				"[{}]: {} => {}\n{:#?}",
				file!(),
				line!(),
				"Failed to access database.",
				why
			);

			return metadata
				.reply(SchnoseResponseData::Message(String::from("Failed to access database.")))
				.await;
		},
		Ok(document) => match document {
			None => {
				return metadata
					.reply(SchnoseResponseData::Message(String::from(
						"You don't have any database entries yet.",
					)))
					.await;
			},
			Some(db_entry) => {
				let embed = CreateEmbed::default()
					.color((116, 128, 194))
					.title(format!("{}'s database entries", &metadata.cmd.user.name))
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

				return metadata.reply(SchnoseResponseData::Embed(embed)).await;
			},
		},
	}
}
