use {
	crate::events::slash_commands::{
		GlobalState,
		InteractionResponseData::{self, *},
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

pub(crate) async fn execute(
	state: &mut GlobalState<'_>,
) -> anyhow::Result<InteractionResponseData> {
	// Defer current interaction since this could take a while
	state.defer().await?;

	let (user_id, blame_user) = match state.get::<u64>("user") {
		Some(user_id) => (user_id, false),
		None => (*state.user.id.as_u64(), true),
	};

	// Search database for the user's Discord User ID
	match state.db.find_one(doc! { "discordID": user_id.to_string() }, None).await {
		// Database connection successful
		Ok(document) => match document {
			// User has an entry in the database
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

				return Ok(Embed(embed));
			},
			None => {
				return Ok(Message(format!(
					"{} a database entry.",
					if blame_user {
						"You don't have"
					} else {
						"The user you specified doesn't have"
					}
				)))
			},
		},
		Err(why) => {
			log::error!("[{}]: {} => {:?}", file!(), line!(), why);
			return Ok(Message("Failed to access database.".into()));
		},
	}
}
