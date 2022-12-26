use {
	crate::{
		prelude::{InteractionResult, SchnoseError},
		events::interactions::InteractionState,
		database::schemas::UserSchema,
	},
	log::{info, warn, error},
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
						info!("Modifying database entry...");
						if let Err(why) = state
							.db
							.find_one_and_update(
								doc! { "discordID": state.user.id.to_string() },
								doc! { "$set": { "mode": &mode_name } },
								None,
							)
							.await
						{
							error!("Failed to update database: {:?}", why);
							return Err(SchnoseError::DBUpdate);
						}
						Ok(format!(
							"Successfully {} mode for <@{}>.{}",
							if mode_name == "none" { "cleared" } else { "set" },
								state.user.id.as_u64(),
							if mode_name == "none" {
								String::new()
							} else {
								format!(
									" New Mode: `{}`",
									mode_name
										.parse::<Mode>()
										.expect("This must be valid at this point. `mode_name` can only be valid or \"none\". The latter is already impossible because of the if-statement above.")
								)
							}).into())
					},
					// user does not yet have an entry => create a new one
					None => {
						warn!("{} doesn't have a database entry.", &state.user.name);
						if mode_name != "none" {
							if let Err(why) = state
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
								error!("Failed to update database: {:?}", why);
								return Err(SchnoseError::DBUpdate);
							}

							if mode_name == "none" {
								return Ok(format!(
									"Successfully cleared mode for <@{}>.",
									state.user.id.as_u64()
								)
								.into());
							}

							Ok(format!(
								"Successfully set mode `{}` for <@{}>.",
								mode_name,
								state.user.id.as_u64()
							)
							.into())
						} else {
							// user doesn't have any database entries but wants to set their mode
							// to "none"
							let err = SchnoseError::Custom(
								"Your tactics confuse and frighten me, sir.".into(),
							);
							info!("{}", &err);
							Err(err)
						}
					},
				},
				Err(why) => {
					error!("Failed to access database: {:?}", why);
					Err(SchnoseError::DBAccess)
				},
			}
		},
		// user did not specify a mode and therefore wants to check their current mode
		None => {
			match state.db.find_one(doc! { "discordID": state.user.id.to_string() }, None).await {
				Ok(document) => match document {
					Some(entry) => match entry.mode {
						Some(mode) if mode != "none" => {
							Ok(format!(
								"Your current mode is set to: `{}`.",
								mode.parse::<Mode>().expect("This must be valid at this point. `mode_name` can only be valid or \"none\". The latter is already impossible because of the if-statement above.")
							).into())
						},
						_ => Err(SchnoseError::Custom("You currently don't have a mode set.".into())),
					},
					None => {
						warn!("{} doesn't have a database entry.", &state.user.name);
						Err(SchnoseError::MissingDBEntry(true))
					},
				},
				Err(why) => {
					error!("Failed to access database: {:?}", why);
					Err(SchnoseError::DBAccess)
				},
			}
		},
	}
}
