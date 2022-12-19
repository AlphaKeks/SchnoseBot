use crate::database::schemas::UserSchema;

use {
	crate::{
		events::interactions::InteractionState,
		prelude::{InteractionResult, SchnoseError},
	},
	log::{info, warn, error},
	bson::doc,
	gokz_rs::prelude::*,
	serenity::{builder::CreateApplicationCommand, model::prelude::command::CommandOptionType},
};

pub(crate) fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	return cmd
		.name("setsteam")
		.description("Save your SteamID in schnose's database.")
		.create_option(|opt| {
			opt.kind(CommandOptionType::String)
				.name("steam_id")
				.description("e.g. `STEAM_1:1:161178172`")
				.required(true)
		});
}

pub(crate) async fn execute(state: &mut InteractionState<'_>) -> InteractionResult {
	// Defer current interaction since this could take a while
	state.defer().await?;

	let steam_id = state.get::<String>("steam_id").expect("This option is marked as `required`.");

	// validate user input
	if !SteamID::test(&steam_id) {
		return Err(SchnoseError::UserInput(steam_id));
	}

	// TODO: normalize SteamIDs to `STEAM_1:1:XXXXXX`
	// -> GlobalAPI normalizes it, and if schnose doesn't, there could be inconsistencies.

	match state.db.find_one(doc! { "discordID": state.user.id.to_string() }, None).await {
		// user has an entry already => update
		Ok(document) => match document {
			Some(_old_entry) => {
				info!("Modifying database entry...");
				match state
					.db
					.find_one_and_update(
						doc! { "discordID": state.user.id.to_string() },
						doc! { "$set": { "steamID": &steam_id } },
						None,
					)
					.await
				{
					Ok(_) => {
						return Ok(format!(
							"Successfully set SteamID `{}` for <@{}>.",
							&steam_id,
							state.user.id.as_u64(),
						)
						.into())
					},
					Err(why) => {
						error!("Failed to update database: {:?}", why);
						Err(SchnoseError::DBUpdate)
					},
				}
			},
			// user does not yet have an entry => create a new one
			None => {
				warn!("{} doesn't have a database entry.", &state.user.name);
				match state
					.db
					.insert_one(
						UserSchema {
							name: state.user.name.clone(),
							discordID: state.user.id.to_string(),
							steamID: Some(steam_id.clone()),
							mode: None,
						},
						None,
					)
					.await
				{
					Ok(_) => {
						return Ok((format!(
							"Successfully set SteamID `{}` for <@{}>.",
							steam_id,
							state.user.id.as_u64()
						))
						.into())
					},
					Err(why) => {
						error!("Failed to update database: {:?}", why);
						Err(SchnoseError::DBUpdate)
					},
				}
			},
		},
		Err(why) => {
			error!("Failed to access database: {:?}", why);
			Err(SchnoseError::DBAccess)
		},
	}
}
