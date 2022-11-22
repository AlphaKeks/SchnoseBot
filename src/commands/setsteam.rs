use {
	crate::{
		db::UserSchema,
		events::slash_command::{InteractionData, InteractionResponseData::Message},
	},
	anyhow::Result,
	gokz_rs::prelude::*,
	bson::doc,
	serenity::{builder::CreateApplicationCommand, model::prelude::command::CommandOptionType},
};

pub fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
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

pub async fn execute(mut ctx: InteractionData<'_>) -> Result<()> {
	ctx.defer().await?;

	let steam_id = ctx.get_string("steam_id").expect("This option is marked as `required`.");

	if !SteamID::test(&steam_id) {
		return ctx.reply(Message("Please enter a valid SteamID.")).await;
	}

	match ctx.db.find_one(doc! { "discordID": ctx.user.id.to_string() }, None).await {
		Ok(document) => match document {
			// update document
			Some(_entry) => {
				match ctx
					.db
					.find_one_and_update(
						doc! { "discordID": ctx.user.id.to_string() },
						doc! { "$set": { "steamID": &steam_id } },
						None,
					)
					.await
				{
					Ok(_) => {
						return ctx
							.reply(Message(&format!(
								"Successfully updated SteamID for <@{}>. New SteamID: `{}`",
								ctx.user.id.as_u64(),
								steam_id,
							)))
							.await
					},
					Err(why) => {
						log::error!("[{}]: {} => {:?}", file!(), line!(), why);
						return ctx.reply(Message("Failed to update databse.")).await;
					},
				}
			},
			// create new document
			None => match ctx
				.db
				.insert_one(
					UserSchema {
						name: ctx.user.name.clone(),
						discordID: ctx.user.id.to_string(),
						steamID: Some(steam_id.clone()),
						mode: None,
					},
					None,
				)
				.await
			{
				Ok(_) => {
					return ctx
						.reply(Message(&format!(
							"Successfully set SteamID `{}` for <@{}>.",
							steam_id,
							ctx.user.id.as_u64()
						)))
						.await
				},
				Err(why) => {
					log::error!("[{}]: {} => {:?}", file!(), line!(), why);
					return ctx.reply(Message("Failed to create database entry.")).await;
				},
			},
		},
		Err(why) => {
			log::error!("[{}]: {} => {}\n{:?}", file!(), line!(), "Failed to acces database.", why);
			return ctx.reply(Message("Failed to access database.")).await;
		},
	}
}
