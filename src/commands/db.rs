use {
	crate::events::slash_commands::{
		GlobalState,
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

pub(crate) async fn execute(mut state: GlobalState<'_>) -> anyhow::Result<()> {
	state.defer().await?;

	let (user_id, blame_user) = match state.get::<u64>("user") {
		Some(user_id) => (user_id, false),
		None => (*state.user.id.as_u64(), true),
	};

	match state.db.find_one(doc! { "discordID": user_id.to_string() }, None).await {
		Ok(document) => match document {
			Some(entry) => {
				let embed = CreateEmbed::default()
					.colour(state.colour)
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

				return state.reply(Embed(embed)).await;
			},
			None => {
				return state
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
			return state.reply(Message("Failed to access database.")).await;
		},
	}
}
