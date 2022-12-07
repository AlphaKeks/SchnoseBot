use {
	crate::{
		db::UserSchema,
		events::slash_commands::{InteractionState, InteractionResponseData::*},
		schnose::{InteractionResult, SchnoseErr},
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

pub(crate) async fn execute(state: &mut InteractionState<'_>) -> InteractionResult {
	// Defer current interaction since this could take a while
	state.defer().await?;

	match state.get::<String>("mode") {
		// user specified a mode and wants to
		// 1. set it for the first time => create new db entry
		// 2. change their current mode => update db entry
		Some(mode_name) => {
			match state.db.find_one(doc! { "discordID": state.user.id.to_string() }, None).await {
				// user has an entry already => update
				Ok(document) => match document {
					Some(_old_entry) => {
						log::info!(
							"[{}]: {} => Modifying database entry\n\n{:?}",
							file!(),
							line!(),
							_old_entry
						);
						match state
							.db
							.find_one_and_update(
								doc! { "discordID": state.user.id.to_string() },
								doc! { "$set": { "mode": &mode_name } },
								None,
							)
							.await
						{
							Ok(_) => {
								return Ok(Message(format!(
									"Successfully {} mode for <@{}>.{}",
									if mode_name == "none" { "cleared" } else { "set" },
									state.user.id.as_u64(),
									if mode_name == "none" {
										String::new()
									} else {
										format!(
											" New Mode: `{}`",
											Mode::from_str(&mode_name)
												.expect("This must be valid at this point. `mode_name` can only be valid or \"none\". The latter is already impossible because of the if-statement above.")
										)
									},
								)));
							},
							Err(why) => {
								log::warn!("[{}]: {} => {:?}", file!(), line!(), why);
								return Err(SchnoseErr::DBUpdate);
							},
						}
					},
					// user does not yet have an entry => create a new one
					None => {
						log::warn!(
							"[{}]: {} => {} doesn't have a database entry.",
							file!(),
							line!(),
							&state.user.name
						);
						if mode_name != "none" {
							match state
								.db
								.insert_one(
									UserSchema {
										name: state.user.name.clone(),
										discordID: state.user.id.to_string(),
										steamID: None,
										mode: Some(mode_name.clone()),
									},
									None,
								)
								.await
							{
								Ok(_) => {
									return if mode_name == "none" {
										Ok(Message(format!(
											"Successfully cleared mode for <@{}>.",
											state.user.id.as_u64()
										)))
									} else {
										Ok(Message(format!(
											"Successfully set mode `{}` for <@{}>.",
											mode_name,
											state.user.id.as_u64()
										)))
									}
								},
								Err(why) => {
									log::warn!("[{}]: {} => {:?}", file!(), line!(), why);
									return Err(SchnoseErr::DBUpdate);
								},
							}
						} else {
							// user doesn't have any database entries but wants to set their mode
							// to "none"
							return Err(SchnoseErr::Custom(
								"Your tactics confuse and frighten me, sir.".into(),
							));
						}
					},
				},
				Err(why) => {
					log::error!("[{}]: {} => {:?}", file!(), line!(), why);
					return Err(SchnoseErr::DBAccess);
				},
			}
		},
		// user did not specify a mode and therefore wants to check their current mode
		None => {
			match state.db.find_one(doc! { "discordID": state.user.id.to_string() }, None).await {
				Ok(document) => match document {
					Some(entry) => match entry.mode {
						Some(mode) if mode != "none" => {
							return Ok(Message(format!(
								"Your current mode is set to: `{}`.",
								Mode::from_str(&mode).expect("This must be valid at this point. `mode_name` can only be valid or \"none\". The latter is already impossible because of the if-statement above.")
							)))
						},
						_ => return Err(SchnoseErr::Custom("You currently don't have a mode set.".into())),
					},
					None => {
						log::warn!(
							"[{}]: {} => {} doesn't have a database entry.",
							file!(),
							line!(),
							&state.user.name
						);
						return Err(SchnoseErr::MissingDBEntry(true));
					},
				},
				Err(why) => {
					log::error!("[{}]: {} => {:?}", file!(), line!(), why);
					return Ok(Message("Failed to access database.".into()));
				},
			}
		},
	}
}
