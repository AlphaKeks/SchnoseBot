use {
	crate::{
		db::UserSchema,
		events::slash_commands::{InteractionData, InteractionResponseData::Message},
	},
	bson::doc,
	gokz_rs::prelude::*,
	serenity::{builder::CreateApplicationCommand, model::prelude::command::CommandOptionType},
};

pub(crate) fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	return cmd
		.name("mode")
		.description("Save your preferred gamemode in schnose's database.")
		.create_option(|opt| {
			opt.kind(CommandOptionType::String)
				.name("mode")
				.description("Choose a mode.")
				.add_string_choice("KZT", "kz_timer")
				.add_string_choice("SKZ", "kz_simple")
				.add_string_choice("VNL", "kz_vanilla")
				.add_string_choice("None", "none")
				.required(false)
		});
}

pub(crate) async fn execute(mut data: InteractionData<'_>) -> anyhow::Result<()> {
	data.defer().await?;

	match data.get_string("mode") {
		// user specified a mode and wants to
		// 1. set it for the first time => create new db entry
		// 2. change their current mode => update db entry
		Some(mode_name) => {
			match data.db.find_one(doc! { "discordID": data.user.id.to_string() }, None).await {
				// user has an entry already => update
				Ok(document) => match document {
					Some(_old_entry) => {
						log::info!(
							"[{}]: {} => Modifying database entry\n\n{:?}",
							file!(),
							line!(),
							_old_entry
						);
						match data
							.db
							.find_one_and_update(
								doc! { "discordID": data.user.id.to_string() },
								doc! { "$set": { "mode": &mode_name } },
								None,
							)
							.await
						{
							Ok(_) => {
								return data
									.reply(Message(&format!(
										"Successfully {} mode for <@{}>.{}",
										if mode_name == "none" { "cleared" } else { "set" },
										data.user.id.as_u64(),
										if mode_name == "none" {
											String::new()
										} else {
											format!(
												" New Mode: `{}`",
												Mode::from_str(&mode_name)
													.expect("This must be valid at this point.")
											)
										},
									)))
									.await;
							},
							Err(why) => {
								log::warn!("[{}]: {} => {:?}", file!(), line!(), why);
								return data.reply(Message("Failed to update database.")).await;
							},
						}
					},
					// user does not yet have an entry => create a new one
					None => {
						log::warn!(
							"[{}]: {} => {} doesn't have a database entry.",
							file!(),
							line!(),
							&data.user.name
						);
						if mode_name != "none" {
							match data
								.db
								.insert_one(
									UserSchema {
										name: data.user.name.clone(),
										discordID: data.user.id.to_string(),
										steamID: None,
										mode: Some(mode_name.clone()),
									},
									None,
								)
								.await
							{
								Ok(_) => {
									return if mode_name == "none" {
										data.reply(Message(&format!(
											"Successfully cleared mode for <@{}>.",
											data.user.id.as_u64()
										)))
										.await
									} else {
										data.reply(Message(&format!(
											"Successfully set mode `{}` for <@{}>.",
											mode_name,
											data.user.id.as_u64()
										)))
										.await
									}
								},
								Err(why) => {
									log::warn!("[{}]: {} => {:?}", file!(), line!(), why);
									return data
										.reply(Message("Failed to create database entry."))
										.await;
								},
							}
						} else {
							// user doesn't have any database entries but wants to set their mode
							// to "none"
							return data
								.reply(Message("Your tactics confuse and frighten me, sir."))
								.await;
						}
					},
				},
				Err(why) => {
					log::error!("[{}]: {} => {:?}", file!(), line!(), why);
					return data.reply(Message("Failed to access database.")).await;
				},
			}
		},
		// user did not specify a mode and therefore wants to check their current mode
		None => {
			match data.db.find_one(doc! { "discordID": data.user.id.to_string() }, None).await {
				Ok(document) => match document {
					Some(entry) => match entry.mode {
						Some(mode) if mode != "none" => {
							return data
								.reply(Message(&format!(
									"Your current mode is set to: `{}`.",
									Mode::from_str(&mode)
										.expect("This must be valid at this point.")
								)))
								.await
						},
						_ => {
							return data
								.reply(Message("You currently don't have a mode set."))
								.await
						},
					},
					None => {
						log::warn!(
							"[{}]: {} => {} doesn't have a database entry.",
							file!(),
							line!(),
							&data.user.name
						);
						return data.reply(Message("You don't have any database entries.")).await;
					},
				},
				Err(why) => {
					log::error!("[{}]: {} => {:?}", file!(), line!(), why);
					return data.reply(Message("Failed to access database.")).await;
				},
			}
		},
	}
}
