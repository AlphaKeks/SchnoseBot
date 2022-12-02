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
		.name("setsteam")
		.description("Save your SteamID in schnose's database.")
		.create_option(|opt| {
			opt.kind(CommandOptionType::String)
				.name("steam_id")
				.description("e.g. `STEAM_1:1:161178172`")
				.required(true)
		});
}

pub(crate) async fn execute(mut data: InteractionData<'_>) -> anyhow::Result<()> {
	data.defer().await?;

	let steam_id = data.get_string("steam_id").expect("This option is marked as `required`.");

	if !SteamID::test(&steam_id) {
		return data
			.reply(Message("Please enter a valid SteamID (e.g. `STEAM_1:1:161178172`)."))
			.await;
	}

	// TODO: normalize steam ids to `STEAM_1:1:XXXXXX`
	// (`STEAM_0:1:XXXXXX` is bad)

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
						doc! { "$set": { "steamID": &steam_id } },
						None,
					)
					.await
				{
					Ok(_) => {
						return data
							.reply(Message(&format!(
								"Successfully set SteamID `{}` for <@{}>.",
								&steam_id,
								data.user.id.as_u64(),
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
				match data
					.db
					.insert_one(
						UserSchema {
							name: data.user.name.clone(),
							discordID: data.user.id.to_string(),
							steamID: Some(steam_id.clone()),
							mode: None,
						},
						None,
					)
					.await
				{
					Ok(_) => {
						return data
							.reply(Message(&format!(
								"Successfully set SteamID `{}` for <@{}>.",
								steam_id,
								data.user.id.as_u64()
							)))
							.await
					},
					Err(why) => {
						log::warn!("[{}]: {} => {:?}", file!(), line!(), why);
						return data.reply(Message("Failed to create database entry.")).await;
					},
				}
			},
		},
		Err(why) => {
			log::error!("[{}]: {} => {:?}", file!(), line!(), why);
			return data.reply(Message("Failed to access database.")).await;
		},
	}
}
