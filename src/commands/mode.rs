use crate::{
	event_handler::interaction_create::{Metadata, SchnoseResponseData},
	util::UserSchema,
};

use bson::doc;

use gokz_rs::prelude::Mode;

use serenity::{builder::CreateApplicationCommand, model::prelude::command::CommandOptionType};

pub fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	cmd.name("mode")
		.description("Save your preferred gamemode in schnose's database.")
		.create_option(|opt| {
			opt.kind(CommandOptionType::String)
				.name("mode")
				.description("Choose a mode.")
				.add_string_choice("KZT", "kz_timer")
				.add_string_choice("SKZ", "kz_simple")
				.add_string_choice("VNL", "kz_vanilla")
				.add_string_choice("None", "none")
				.required(true)
		})
}

pub async fn run(metadata: Metadata, collection: &mongodb::Collection<UserSchema>) {
	// sanitize user input
	let user_input = match metadata.opts.get_string("mode") {
		Some(mode_str) => mode_str,
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
							steamID: None,
							mode: if user_input == "none" {
								None
							} else {
								Some(user_input.clone())
							},
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
						return metadata.reply(SchnoseResponseData::Message(if user_input == "none" {
							format!(
								"Successfully cleared Mode for <@{}>.",
								metadata.cmd.user.id.as_u64()
							)
						} else {
							let mode = match Mode::from_str(&user_input) {
										Ok(mode) => mode,
										Err(_) => unreachable!("can only be valid; the if statement above ensures that the input is not `none`.")
									};
							format!(
								"Successfully set Mode `{}` for <@{}>.",
								mode.to_fancy(),
								metadata.cmd.user.id.as_u64()
							)
						})).await;
					},
				}
			},
			// user already has a database entry, so we update it
			Some(_) => {
				// try to update database entry
				match collection
					.find_one_and_update(
						doc! { "discordID": metadata.cmd.user.id.to_string() },
						doc! { "$set": { "mode": &user_input } },
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
								"Successfully {}",
								if user_input == "none" {
									format!("cleared Mode for <@{}>", metadata.cmd.user.id.as_u64())
								} else {
									let mode = match Mode::from_str(&user_input) {
										Ok(mode) => mode,
										Err(_) => unreachable!(
										"can only be valid or `none` => `none` already covered by the if statement above"
									),
									};
									format!(
										"set Mode `{}` for <@{}>",
										mode.to_fancy(),
										metadata.cmd.user.id.as_u64()
									)
								}
							)))
							.await;
					},
				}
			},
		},
	}
}
