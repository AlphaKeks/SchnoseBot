use {
	super::handle_err,
	std::collections::BTreeMap,
	log::trace,
	poise::serenity_prelude::{CollectComponentInteraction, InteractionResponseType},
};

/// Help Menu
#[poise::command(slash_command, on_error = "handle_err", ephemeral)]
pub async fn help(ctx: crate::Context<'_>) -> Result<(), crate::SchnoseError> {
	trace!("HELP ({})", &ctx.author().tag());

	let pages: BTreeMap<&str, (&str, &str)> = BTreeMap::from_iter([
		(
			"h_apistatus",
			(
				"Check the GlobalAPI's current health status.",
				"
Wanna know if the API is down? `/apistatus` will tell you how healthy the API currently is. It uses [this website](https://health.global-api.com/endpoints/_globalapi) btw, if you want to check for yourself!
				",
			)
		),
		(
			"h_bmaptop",
			(
				"Check the top 100 records on a bonus.",
				"
Will fetch the top 100 records on a bonus course of the map you specified. The bot will send embeds with a maximum of 12 entries per page, displaying the player names and times. You can specify the following paramters:

∙ `map_name`: The map of the bonus (any of [these](https://maps.global-api.com/mapcycles/gokz.txt))
∙ `mode`: Filter by Mode (KZT/SKZ/VNL)
∙ `runtype`: TP/PRO
∙ `course`: Which bonus course you want to check (e.g. `3`)

`mode`, `runtype` and `course` are _optional_.

If you don't specify `mode`, the bot will try to look for a database entry with your user id. If it can't find one, or if you haven't saved your mode via `/mode`, the command will fail.

If you don't specify `runtype`, it will default to `PRO`.

If you don't specify `course`, it will default to `1`.
				"
			)
		),
		(
			"h_bpb",
			(
				"Check a player's personal best on a bonus.",
				"
Will fetch a given player's personal best on a bonus course of the map you specified. The bot will send an embed with both the TP and PRO PBs, if there are any. You can specify the following paramters:

∙ `map_name`: The map of the bonus (any of [these](https://maps.global-api.com/mapcycles/gokz.txt))
∙ `mode`: Filter by Mode (KZT/SKZ/VNL)
∙ `course`: Which bonus course you want to check (e.g. Bonus **1**)
∙ `player`: You can either @mention someone, pass in a SteamID, a player's name, or leave it empty. The bot will try its best to make sense of the input!

`mode`, `course` and `player` are _optional_.

If you don't specify `mode`, the bot will try to look for a database entry with your user id. If it can't find one, or if you haven't saved your mode via `/mode`, the command will fail.

If you don't specify `course`, it will default to `1`.

If you don't specify `player`, the bot will attempt to look for a database entry with your user id. If if can't find one, or if you haven't saved your SteamID via `/setsteam`, the command will fail.
				"
			)
		),
		(
			"h_btop",
			(
				"Check the top 100 bonus record holders.",
				"
Will fetch the a leaderboard of top bonus record holders. You can specify the following paramters:

∙ `mode`: Filter by Mode (KZT/SKZ/VNL)
∙ `runtype`: TP/PRO

Both parameters are optional.

If you don't specify `mode`, the bot will try to look for a database entry with your user id. If it can't find one, or if you haven't saved your mode via `/mode`, the command will fail.

If you don't specify `runtype`, it will default to `PRO`.
				"
			)
		),
		(
			"h_bwr",
			(
				"Check the world record on a bonus.",
				"
Will fetch the world record on a bonus course of the map you specified. The bot will send an embed with both the TP and PRO WRs, if there are any. You can specify the following paramters:

∙ `map_name`: The map of the bonus (any of [these](https://maps.global-api.com/mapcycles/gokz.txt))
∙ `mode`: Filter by Mode (KZT/SKZ/VNL)
∙ `course`: Which bonus course you want to check (e.g. Bonus **1**)

`mode` and `course` are _optional_.

If you don't specify `mode`, the bot will try to look for a database entry with your user id. If it can't find one, or if you haven't saved your mode via `/mode`, the command will fail.

If you don't specify `course`, it will default to `1`.
				"
			)
		),
		(
			"h_db",
			(
				"Check you current database entries.",
				"Will query the database with your user id and retrieve the entries, if there are any. By default the bot's response will only be visible to you, but you can tell it to send a public message if you want to show your data to others."
			)
		),
		(
			"h_invite",
			(
				"Invite the bot to your server!",
				"Will send a message with an invite link for this bot."
			)
		),
		(
			"h_map",
			(
				"Will fetch detailed information about a given map.",
				"
Will fetch data about the map you specified from both the GlobalAPI and [KZ:GO](https://kzgo.eu/). You can specify the following paramters:

∙ `map_name`: The map you want more information about (any of [these](https://maps.global-api.com/mapcycles/gokz.txt))
				"
			)
		),
		(
			"h_maptop",
			(
				"Check the top 100 records on a map.",
				"
Will fetch the top 100 records on the main course of the map you specified. The bot will send embeds with a maximum of 12 entries per page, displaying the player names and times. You can specify the following paramters:

∙ `map_name`: The map of the bonus (any of [these](https://maps.global-api.com/mapcycles/gokz.txt))
∙ `mode`: Filter by Mode (KZT/SKZ/VNL)
∙ `runtype`: TP/PRO

`mode` and `runtype` are _optional_.

If you don't specify `mode`, the bot will try to look for a database entry with your user id. If it can't find one, or if you haven't saved your mode via `/mode`, the command will fail.

If you don't specify `runtype`, it will default to `PRO`.
				"
			)
		),
		(
			"h_mode",
			(
				"Save your favorite mode for later use.",
				"Many commands require you to specify a mode. To get around this, you can save a fallback option in the bot's database via `/mode`. Use it once, and never specify a mode again! Keep in mind that this is meant as a fallback; if you specify the mode on a command manually, that will have priority."
			)
		),
		(
			"h_nocrouch",
			(
				"Approximate the distance of a nocrouch jump.",
				"
Ever hit a sick LJ but forgot to crouch, and thought \"Man, that would've been a 280 if I crouched!!!\"? There is a highly complicated math formula to calculate an approximation of your jump, given some data...

`final_distance = distance + (max_speed / 128) * 4`

This formula is very optimistic, assuming two things:
∙ You didn't lose _any_ speed on your last strafe
∙ You had the perfect airpath on your last strafe

It's only an approximation.
				"
			)
		),
		(
			"h_pb",
			(
				"Check a player's personal best on a map.",
				"
Will fetch a given player's personal best on the main course of the map you specified. The bot will send an embed with both the TP and PRO PBs, if there are any. You can specify the following paramters:

∙ `map_name`: any of [these](https://maps.global-api.com/mapcycles/gokz.txt)
∙ `mode`: Filter by Mode (KZT/SKZ/VNL)
∙ `player`: You can either @mention someone, pass in a SteamID, a player's name, or leave it empty. The bot will try its best to make sense of the input!

`mode` and `player` are _optional_.

If you don't specify `mode`, the bot will try to look for a database entry with your user id. If it can't find one, or if you haven't saved your mode via `/mode`, the command will fail.

If you don't specify `player`, the bot will attempt to look for a database entry with your user id. If if can't find one, or if you haven't saved your SteamID via `/setsteam`, the command will fail.
				"
			)
		),
		(
			"h_random",
			(
				"Generate a random KZ map.",
				"Generates a random KZ map along with its tier. You _can_ filter by tier, if you want. If you don't specify a tier, the bot will ignore the tier when choosing a map."
			)
		),
		(
			"h_recent",
			(
				"Check a player's most recently set personal best.",
				"
Will fetch a given player's most recently set personal best regardless of mode; it will also give you links to watch/download the replay if it is available. Command will ignore bonuses. You can specify the following paramter:

∙ `player`: You can either @mention someone, pass in a SteamID, a player's name, or leave it empty. The bot will try its best to make sense of the input!

`player` is _optional_.

If you don't specify `player`, the bot will attempt to look for a database entry with your user id. If if can't find one, or if you haven't saved your SteamID via `/setsteam`, the command will fail.
				"
			)
		),
		(
			"h_report",
			(
				"Report issues with the bot or suggest changes!",
				"If you have any complaints or suggestions, feel free to submit them here! Alternatively you can submit an [issue on GitHub](https://github.com/AlphaKeks/SchnoseBot/issues) or send <@291585142164815873> a DM!"
			)
		),
		(
			"h_setsteam",
			(
				"Save your SteamID for later use.",
				"Many commands require you to specify a player. To get around this, you can save your own SteamID as a fallback option in the bot's database via `/setsteam`. Use it once, and never specify a `player` again! Keep in mind that this is meant as a fallback; if you specify the player on a command manually, that will have priority."
			)
		),
		(
			"h_top",
			(
				"Check the top 100 world record holders.",
				"
Will fetch the a leaderboard of top world record holders. You can specify the following paramters:

∙ `mode`: Filter by Mode (KZT/SKZ/VNL)
∙ `runtype`: TP/PRO

Both parameters are optional.

If you don't specify `mode`, the bot will try to look for a database entry with your user id. If it can't find one, or if you haven't saved your mode via `/mode`, the command will fail.

If you don't specify `runtype`, it will default to `PRO`.
				"
			)
		),
		(
			"h_unfinished",
			(
				"Check which maps a player still has to complete.",
				"
Will fetch all maps a given player hasn't finished yet. The bot will display the first 10 maps, and tell you the total amount. You can specify the following paramters:

∙ `mode`: Filter by Mode (KZT/SKZ/VNL)
∙ `runtype`: TP/PRO
∙ `tier`: You can either @mention someone, pass in a SteamID, a player's name, or leave it empty. The bot will try its best to make sense of the input!
∙ `player`: You can either @mention someone, pass in a SteamID, a player's name, or leave it empty. The bot will try its best to make sense of the input!

None of the options is required.

If you don't specify `mode`, the bot will try to look for a database entry with your user id. If it can't find one, or if you haven't saved your mode via `/mode`, the command will fail.

If you don't specify `runtype`, it will default to `PRO`.

If you don't specify `tier`, the bot will ignore it when fetching the maps.

If you don't specify `player`, the bot will attempt to look for a database entry with your user id. If if can't find one, or if you haven't saved your SteamID via `/setsteam`, the command will fail.
				"
			)
		),
		(
			"h_wr",
			(
				"Check the world record on a map.",
				"
Will fetch the world record on the main course of the map you specified. The bot will send an embed with both the TP and PRO WRs, if there are any. You can specify the following paramters:

∙ `map_name`: The map of the bonus (any of [these](https://maps.global-api.com/mapcycles/gokz.txt))
∙ `mode`: Filter by Mode (KZT/SKZ/VNL)

`mode` is _optional_.

If you don't specify `mode`, the bot will try to look for a database entry with your user id. If it can't find one, or if you haven't saved your mode via `/mode`, the command will fail.
				"
			)
		),
	]);

	let ctx_id = ctx.id();

	let pages2 = pages.clone();

	// Send the embed with the first page as content
	ctx.send(|reply| {
		reply
			.embed(|e| {
				e.description(
					"
First of all, thank you for using this bot! I always appreciate suggestions and bug reports, so if you have anything, feel free to reach out via `/report`, DM <@291585142164815873> or submit an [issue on GitHub](https://github.com/AlphaKeks/SchnoseBot/issues).

This bot features most commands you already know from ingame like `/pb`, `/wr` or `/maptop`, as well as a bunch of other useful commands. Check the GlobalAPI's health via `/apistatus` or look at your most recently set PB with `/recent`! If you need any detailed information about a specific command, click on the dropdown below and select the command you need help with.

To get started, set your SteamID with `/setsteam` and your favorite game mode with `/mode`. These are not required, but a lot of commands need those parameters; which is why you can save them in the bot's database for later use.
					"
					)
				}
			)
			.components(|components| {
				components.create_action_row(|row| {
					row.create_select_menu(|menu| {
						menu.custom_id(ctx_id).options(|o| {
							for (value, (short_description, _)) in pages {
								o.create_option(|o| {
									let label = value
										.strip_prefix("h_")
										.expect("Every value should have this prefix.");

									o.label(format!("/{}", label))
										.value(value)
										.description(short_description)
								});
							}
							o
						})
					})
				})
			})
	})
	.await?;

	// Loop through incoming interactions
	while let Some(selection) = CollectComponentInteraction::new(ctx)
		// Filter by correct `custom_id`
		.filter(move |selection| selection.data.custom_id == ctx_id.to_string())
		// Listen for any changes for 10 minutes
		.timeout(std::time::Duration::from_secs(600))
		.await
	{
		let choice = &selection.data.values[0];
		selection
			.create_interaction_response(ctx, |response| {
				response.kind(InteractionResponseType::UpdateMessage).interaction_response_data(
					|data| {
						data.embed(|e| {
							let (_, long_description) =
								pages2.get(choice.as_str()).expect("Missing help page.");
							e.title(format!(
								"/{}",
								choice.strip_prefix("h_").expect("This should have a prefix.")
							))
							.description(long_description)
						})
					},
				)
			})
			.await?;
	}

	Ok(())
}
