use {
	crate::{
		events::interactions::InteractionState,
		prelude::{InteractionResult, SchnoseError},
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

pub(crate) async fn execute(state: &mut InteractionState<'_>) -> InteractionResult {
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
						&entry.steamID.unwrap_or_else(|| String::from("none")),
						&entry.mode.unwrap_or_else(|| String::from("none")),
					))
					.to_owned();

				Ok(embed.into())
			},
			None => Err(SchnoseError::MissingDBEntry(blame_user)),
		},
		Err(why) => {
			log::error!("[{}]: {} => {:?}", file!(), line!(), why);
			Err(SchnoseError::DBAccess)
		},
	}
}
