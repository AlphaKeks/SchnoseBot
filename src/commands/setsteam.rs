use {
	crate::{
		db::UserSchema,
		events::slash_commands::{GlobalState, InteractionResponseData::Message},
	},
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

pub(crate) async fn execute(mut state: GlobalState<'_>) -> anyhow::Result<()> {
	state.defer().await?;

	let steam_id = state.get::<String>("steam_id").expect("This option is marked as `required`.");

	if !SteamID::test(&steam_id) {
		return state
			.reply(Message("Please enter a valid SteamID (e.g. `STEAM_1:1:161178172`)."))
			.await;
	}

	// TODO: normalize steam ids to `STEAM_1:1:XXXXXX`
	// (`STEAM_0:1:XXXXXX` is bad)

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
						doc! { "$set": { "steamID": &steam_id } },
						None,
					)
					.await
				{
					Ok(_) => {
						return state
							.reply(Message(&format!(
								"Successfully set SteamID `{}` for <@{}>.",
								&steam_id,
								state.user.id.as_u64(),
							)))
							.await;
					},
					Err(why) => {
						log::warn!("[{}]: {} => {:?}", file!(), line!(), why);
						return state.reply(Message("Failed to update database.")).await;
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
						return state
							.reply(Message(&format!(
								"Successfully set SteamID `{}` for <@{}>.",
								steam_id,
								state.user.id.as_u64()
							)))
							.await
					},
					Err(why) => {
						log::warn!("[{}]: {} => {:?}", file!(), line!(), why);
						return state.reply(Message("Failed to create database entry.")).await;
					},
				}
			},
		},
		Err(why) => {
			log::error!("[{}]: {} => {:?}", file!(), line!(), why);
			return state.reply(Message("Failed to access database.")).await;
		},
	}
}
