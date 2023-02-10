use {
	crate::{error::Error, Context, State},
	log::trace,
	poise::serenity_prelude::{CollectComponentInteraction, InteractionResponseType},
	std::{collections::BTreeMap, time::Duration},
};

/// Help Menu
///
/// First of all, thank you for using this bot! I always appreciate suggestions and bug reports, \
/// so if you have anything, feel free to reach out via:
/// 1. `/report`
/// 2. DM to <@291585142164815873>
/// 3. an Issue on [GitHub](https://github.com/AlphaKeks/SchnoseBot/issues)
///
/// This bot features most commands you already know from ingame such as
/// - `/pb`
/// - `/wr`
/// - `/maptop`
/// as well as a bunch of other utility commands.
///
/// To get started, type `/` and click on schnose's icon on the left to see all available \
/// commands, or scroll through this help menu. A lot of commands will have `mode` or `player` as \
/// a possible command argument (e.g. `/pb`) since the bot doesn't know _which_ PB to look up. \
/// It's gonna try to guess as best as it can, but to get the best results, you should save your \
/// `SteamID` and your preferred mode in the bot's database. You can do that with `/setsteam` and \
/// `/mode`. Those will then be used as fallback options.
#[poise::command(slash_command, ephemeral, on_error = "Error::handle_command")]
pub async fn help(ctx: Context<'_>) -> Result<(), Error> {
	trace!("[/help ({})]", ctx.author().tag());

	let commands = ctx
		.framework()
		.options()
		.commands
		.iter()
		.filter_map(|command| {
			Some((
				command.name.clone(),
				(command.description.as_ref()?.to_owned(), command.help_text?()),
			))
		})
		.collect::<BTreeMap<_, _>>();

	let ctx_id = ctx.id();

	ctx.send(|reply| {
		let (description_short, description_long) = commands
			.get("help")
			.expect("The /help command should have a help text.");
		reply
			.embed(|e| {
				e.color(ctx.color())
					.title(description_short)
					.description(description_long)
					.footer(|f| {
						f.text(ctx.schnose())
							.icon_url(ctx.icon())
					})
			})
			.components(|c| {
				c.create_action_row(|row| {
					row.create_select_menu(|menu| {
						menu.custom_id(ctx_id).options(|o| {
							for (cmd_name, (description_short, _)) in &commands {
								o.create_option(|o| {
									o.label(format!("/{cmd_name}"))
										.value(cmd_name)
										.description(description_short)
								});
							}
							o
						})
					})
				})
			})
	})
	.await?;

	while let Some(interaction) = CollectComponentInteraction::new(ctx)
		.filter(move |interaction| interaction.data.custom_id == ctx_id.to_string())
		.timeout(Duration::from_secs(600))
		.await
	{
		let choice = &interaction.data.values[0];

		interaction
			.create_interaction_response(ctx, |response| {
				response
					.kind(InteractionResponseType::UpdateMessage)
					.interaction_response_data(|data| {
						data.embed(|e| {
							let (description_short, description_long) =
								commands.get(choice.as_str()).map_or(
									(format!("/{choice}"), String::new()),
									|(description_short, description_long)| {
										(
											format!(
												"[/{choice}]: {}",
												description_short.to_owned()
											),
											description_long.to_owned(),
										)
									},
								);

							e.color(ctx.color())
								.title(description_short)
								.description(description_long)
								.footer(|f| {
									f.text(ctx.schnose())
										.icon_url(ctx.icon())
								})
						})
					})
			})
			.await?;
	}

	Ok(())
}
