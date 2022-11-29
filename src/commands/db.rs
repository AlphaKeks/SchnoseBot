use {
	crate::events::slash_commands::{
		InteractionData,
		InteractionResponseData::{Message, Embed},
	},
	bson::doc,
	serenity::{
		builder::{CreateApplicationCommand, CreateEmbed},
		model::prelude::command::CommandOptionType,
	},
};

pub(crate) fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	return cmd
		.name("db")
		.description("Check a user's current database entries.")
		.create_option(|opt| {
			opt.kind(CommandOptionType::User)
				.name("user")
				.description("Specify a user.")
				.required(false)
		});
}

pub(crate) async fn execute(mut data: InteractionData<'_>) -> anyhow::Result<()> {
	data.defer().await?;

	let (user_id, blame_user) = match data.get_user("user") {
		Some(user_id) => (user_id, false),
		None => (*data.user.id.as_u64(), true),
	};

	match data.db.find_one(doc! { "discordID": user_id.to_string() }, None).await {
		Ok(document) => match document {
			Some(entry) => {
				let embed = CreateEmbed::default()
					.colour(data.colour)
					.title(format!("{}'s database entries", &entry.name))
					.description(format!(
						r#"
> name: {}
> discordID: {}
> steamID: {}
> mode: {}
						"#,
						&entry.name,
						&entry.discordID,
						&entry.steamID.unwrap_or(String::from("none")),
						&entry.mode.unwrap_or(String::from("none")),
					))
					.to_owned();

				return data.reply(Embed(embed)).await;
			},
			None => {
				return data
					.reply(Message(&format!(
						"{} a database entry.",
						if blame_user {
							"You don't have"
						} else {
							"The user you specified doesn't have"
						}
					)))
					.await
			},
		},
		Err(why) => {
			log::error!("[{}]: {} => {:?}", file!(), line!(), why);
			return data.reply(Message("Failed to access database.")).await;
		},
	}
}
