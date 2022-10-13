use crate::util::UserSchema;
use bson::doc;
use gokz_rs::prelude::*;
use serenity::builder::CreateApplicationCommand;
use serenity::json::Value;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::CommandDataOption;
use serenity::model::user::User;

use crate::SchnoseCommand;

pub fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	cmd.name("mode")
		.description("Save your preferred gamemode in schnose's database.")
		.create_option(|opt| {
			opt.kind(CommandOptionType::String)
				.name("mode")
				.description("Specify a mode.")
				.add_string_choice("KZT", "kz_timer")
				.add_string_choice("SKZ", "kz_simple")
				.add_string_choice("VNL", "kz_vanilla")
				.add_string_choice("None", "none")
				.required(true)
		})
}

pub async fn run(
	user: &User,
	opts: &[CommandDataOption],
	mongo_client: &mongodb::Client,
) -> SchnoseCommand {
	let database = mongo_client
		.database("gokz")
		.collection::<UserSchema>("users");

	let mut input = None;

	for opt in opts {
		match opt.name.as_str() {
			"mode" => match &opt.value {
				Some(val) => match val.to_owned() {
					Value::String(mode_val) => input = Some(Mode::from(mode_val)),
					_ => {
						return SchnoseCommand::Message(String::from(
							"Failed to deserialize input mode.",
						))
					}
				},
				None => unreachable!("Failed to access required command option"),
			},
			_ => (),
		}
	}

	match database
		.find_one(doc! { "discordID": user.id.to_string() }, None)
		.await
	{
		// create new one
		Err(_) => return SchnoseCommand::Message(String::from("Failed to access database.")),

		// update
		Ok(document) => match document {
			Some(_) => {
				match database
					.find_one_and_update(
						doc! { "discordID": user.id.to_string() },
						doc! {
							"$set": { "mode": match input.clone() {
									Some(mode) => Some(mode.as_str().to_string()),
									None => None
								}
							}
						},
						None,
					)
					.await
				{
					Ok(_) => {
						return SchnoseCommand::Message(format!(
							"Successfully updated mode for `{}`. New mode: `{}`",
							user.name,
							match input {
								Some(mode) => mode.fancy_short(),
								None => String::from("none"),
							}
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
				match database
					.insert_one(
						UserSchema {
							name: user.name.clone(),
							discordID: user.id.to_string(),
							steamID: None,
							mode: match input.clone() {
								Some(mode) => Some(mode.as_str().to_owned()),
								None => None,
							},
						},
						None,
					)
					.await
				{
					Ok(_) => {
						return SchnoseCommand::Message(format!(
							"Successfully set mode `{}` for `{}`.",
							match input {
								Some(mode) => mode.fancy_short(),
								None => String::from("none"),
							},
							user.name
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
