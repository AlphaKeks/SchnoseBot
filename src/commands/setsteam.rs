use crate::util::{is_steamid, UserSchema};
use bson::doc;
use serenity::builder::CreateApplicationCommand;
use serenity::json::Value;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::CommandDataOption;
use serenity::model::user::User;

use crate::SchnoseCommand;

pub fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	cmd.name("setsteam")
		.description("Save your steamID in schnose's database")
		.create_option(|opt| {
			opt.kind(CommandOptionType::String)
				.name("steam_id")
				.description("e.g. STEAM_1:1:161178172")
				.required(true)
		})
}

pub async fn run(
	user: &User,
	opts: &[CommandDataOption],
	mongo_client: &mongodb::Client,
) -> SchnoseCommand {
	let collection = mongo_client
		.database("gokz")
		.collection::<UserSchema>("users");

	let mut input = None;

	for opt in opts {
		match opt.name.as_str() {
			"steam_id" => match &opt.value {
				Some(val) => match val.to_owned() {
					Value::String(str) => {
						if is_steamid(&str) {
							input = Some(str);
						} else {
							return SchnoseCommand::Message(String::from(
								"Please input a valid steamID.",
							));
						}
					}
					_ => {
						return SchnoseCommand::Message(String::from(
							"Please input a valid steamID.",
						))
					}
				},
				None => unreachable!("Failed to access required command option"),
			},
			_ => (),
		}
	}

	let input = match input {
		None => return SchnoseCommand::Message(String::from("Please input a valid steamID.")),
		Some(s) => s,
	};

	match collection
		.find_one(doc! { "discordID": user.id.to_string() }, None)
		.await
	{
		// create new one
		Err(_) => return SchnoseCommand::Message(String::from("Failed to access database.")),

		// update
		Ok(document) => match document {
			Some(_) => {
				match collection
					.find_one_and_update(
						doc! { "discordID": user.id.to_string() },
						doc! {
							"$set": { "steamID": input.clone() }
						},
						None,
					)
					.await
				{
					Ok(_) => {
						return SchnoseCommand::Message(format!(
							"Successfully updated steamID for `{}`. New steamID: `{}`",
							user.name, input
						))
					}
					_ => {
						return SchnoseCommand::Message(String::from(
							"Failed to update database entry.",
						))
					}
				}
			}
			None => {
				match collection
					.insert_one(
						UserSchema {
							name: user.name.clone(),
							discordID: user.id.to_string(),
							steamID: Some(input.clone()),
							mode: None,
						},
						None,
					)
					.await
				{
					Ok(_) => {
						return SchnoseCommand::Message(format!(
							"Successfully set steamID `{}` for `{}`.",
							input, user.name
						))
					}
					_ => {
						return SchnoseCommand::Message(String::from(
							"Failed to create database entry.",
						))
					}
				}
			}
		},
	}
}
