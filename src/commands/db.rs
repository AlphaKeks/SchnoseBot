use {
	crate::events::slash_command::{
		InteractionData,
		InteractionResponseData::{Message, Embed},
	},
	anyhow::Result,
	bson::doc,
	serenity::{
		builder::{CreateApplicationCommand, CreateEmbed},
		model::prelude::command::CommandOptionType,
	},
};

pub fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
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

pub async fn execute(mut ctx: InteractionData<'_>) -> Result<()> {
	ctx.defer().await?;

	let (user_id, target) = match ctx.get_user("user") {
		Some(h) => (h, true),
		None => (ctx.user.id.as_u64().to_owned(), false),
	};

	match ctx.db.find_one(doc! { "discordID": user_id.to_string() }, None).await {
		Ok(document) => match document {
			Some(entry) => {
				let embed = CreateEmbed::default()
					.color((116, 128, 194))
					.title(format!("<@{}>'s database entries", user_id))
					.description(format!(
						"
> discordID: {}
> steamID: {}
> mode: {}
",
						entry.discordID,
						entry.steamID.unwrap_or(String::from("none")),
						entry.mode.unwrap_or(String::from("none"))
					))
					.to_owned();

				return ctx.reply(Embed(embed)).await;
			},
			None => {
				return ctx
					.reply(Message(&format!(
						"{} have any database entries yet.",
						if target { "The specified user doesn't" } else { "You don't" }
					)))
					.await
			},
		},
		Err(why) => {
			log::error!("[{}]: {} => {}\n{:?}", file!(), line!(), "Failed to acces database.", why);
			return ctx.reply(Message("Failed to access database.")).await;
		},
	}
}
