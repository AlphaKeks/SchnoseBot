use crate::{
	event_handler::interaction_create::{Metadata, SchnoseResponseData},
	util::UserSchema,
};

use bson::doc;

use gokz_rs::prelude::SteamID;

use serenity::{builder::CreateApplicationCommand, model::prelude::command::CommandOptionType};

pub fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	cmd.name("setsteam")
		.description("Save your SteamID in schnose's database.")
		.create_option(|opt| {
			opt.kind(CommandOptionType::String)
				.name("steam_id")
				.description("e.g. `STEAM_1:1:161178172`")
				.required(true)
		})
}

pub async fn run(metadata: Metadata, collection: &mongodb::Collection<UserSchema>) {
	// sanitize user input
	let mut user_input = match metadata.opts.get_string("steam_id") {
		Some(steam_id) => {
			if SteamID::test(&steam_id) {
				steam_id
			} else {
				return metadata
					.reply(SchnoseResponseData::Message(String::from(
						"Please enter a valid SteamID.",
					)))
					.await;
			}
		},
		None => unreachable!("option is required"),
	};

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
			// user does not have a database entry yet, so we create a new one
			None => {
				match collection
					.insert_one(
						UserSchema {
							name: metadata.cmd.user.name.clone(),
							discordID: metadata.cmd.user.id.to_string(),
							steamID: Some(user_input.clone()),
							mode: None,
						},
						None,
					)
					.await
				{
					Err(why) => {
						log::error!(
							"[{}]: {} => {}\n{:#?}",
							file!(),
							line!(),
							"Failed to create new database entry.",
							why
						);

						return metadata
							.reply(SchnoseResponseData::Message(String::from(
								"Failed to create new database entry.",
							)))
							.await;
					},
					Ok(_) => {
						return metadata
							.reply(SchnoseResponseData::Message(format!(
								"Successfully set SteamID `{}` for <@{}>.",
								user_input,
								metadata.cmd.user.id.as_u64()
							)))
							.await;
					},
				}
			},
			// user already has a database entry, so we update it
			Some(_) => {
				// normalize SteamID
				if user_input.starts_with("STEAM_0") {
					user_input.replace_range(0..7, "STEAM_1");
				}

				// try to update database entry
				match collection
					.find_one_and_update(
						doc! { "discordID": metadata.cmd.user.id.to_string() },
						doc! { "$set": { "steamID": &user_input } },
						None,
					)
					.await
				{
					Err(why) => {
						log::error!(
							"[{}]: {} => {}\n{:#?}",
							file!(),
							line!(),
							"Failed to update database entry.",
							why
						);

						return metadata
							.reply(SchnoseResponseData::Message(String::from(
								"Failed to update database entry.",
							)))
							.await;
					},
					Ok(_) => {
						return metadata
							.reply(SchnoseResponseData::Message(format!(
								"Successfully updated SteamID for <@{}>. New SteamID: `{}`",
								metadata.cmd.user.id.as_u64(),
								user_input
							)))
							.await;
					},
				}
			},
		},
	}
}
