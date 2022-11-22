use {
	crate::{
		db::UserSchema,
		events::slash_command::{InteractionData, InteractionResponseData::Message},
	},
	anyhow::Result,
	bson::doc,
	gokz_rs::prelude::*,
	serenity::{builder::CreateApplicationCommand, model::prelude::command::CommandOptionType},
};

pub fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
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

pub async fn execute(mut ctx: InteractionData<'_>) -> Result<()> {
	ctx.defer().await?;

	match ctx.db.find_one(doc! { "discordID": ctx.user.id.to_string() }, None).await {
		Ok(document) => match document {
			// user already has a database entry
			Some(entry) => match ctx.get_string("mode") {
				// user wants to replace his current mode
				Some(mode) => match ctx
					.db
					.find_one_and_update(
						doc! { "discordID": ctx.user.id.to_string() },
						doc! { "$set": { "mode": &mode } },
						None,
					)
					.await
				{
					Ok(_) => {
						if mode == "none" {
							return ctx
								.reply(Message(&format!(
									"Successfully cleared mode for <@{}>",
									ctx.user.id.as_u64()
								)))
								.await;
						} else {
							return ctx
								.reply(Message(&format!(
									"Successfully updated mode for <@{}>. New Mode: `{}`",
									ctx.user.id.as_u64(),
									Mode::from_str(&mode)
										.expect("`mode` should be valid at this point.")
										.to_fancy()
								)))
								.await;
						}
					},
					Err(why) => {
						log::error!(
							"[{}]: {} => {}\n{:?}",
							file!(),
							line!(),
							"Failed to update database entry",
							why
						);
						return ctx.reply(Message("Failed to update database entry.")).await;
					},
				},
				// user wants to check their current entry
				None => {
					return ctx
						.reply(Message(&format!(
							"Your current mode preference is set to: `{}`",
							match entry.mode {
								Some(mode) =>
									if mode == "none" {
										String::from("none")
									} else {
										Mode::from_str(&mode)
											.expect("`mode` should be valid at this point.")
											.to_fancy()
									},
								None => String::from("none"),
							}
						)))
						.await
				},
			},
			// user doesn't have a database entry yet
			None => match ctx.get_string("mode") {
				// user wants to create a new entry
				Some(mode) => match ctx
					.db
					.insert_one(
						UserSchema {
							name: ctx.user.name.clone(),
							discordID: ctx.user.id.to_string(),
							steamID: None,
							mode: if mode == "none" { None } else { Some(mode.clone()) },
						},
						None,
					)
					.await
				{
					Ok(_) => {
						return ctx
							.reply(Message(&format!(
								"Successfully set mode `{}` for <@{}>",
								Mode::from_str(&mode)
									.expect("`mode` should be valid at this point.")
									.to_fancy(),
								ctx.user.id.as_u64()
							)))
							.await
					},
					Err(why) => {
						log::error!(
							"[{}]: {} => {}\n{:?}",
							file!(),
							line!(),
							"Failed to insert into database.",
							why
						);
						return ctx.reply(Message("Failed to insert into database.")).await;
					},
				},
				// user didn't specify a mode and also doesn't have
				// one yet
				None => return ctx.reply(Message("You don't have a mode preference set.")).await,
			},
		},
		Err(why) => {
			log::error!("[{}]: {} => {}\n{:?}", file!(), line!(), "Failed to acces database.", why);
			return ctx.reply(Message("Failed to access database.")).await;
		},
	}
}
