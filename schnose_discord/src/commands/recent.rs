use {
	super::{handle_err, Target, GLOBAL_MAPS},
	crate::{formatting, GlobalStateAccess, SchnoseError},
	gokz_rs::{prelude::*, GlobalAPI},
	log::trace,
	poise::serenity_prelude::{CollectComponentInteraction, CreateEmbed},
	std::time::Duration,
};

/// Check a player's most recently set personal best.
#[poise::command(slash_command, on_error = "handle_err", global_cooldown = 30, user_cooldown = 60)]
pub async fn recent(
	ctx: crate::Context<'_>,
	#[description = "The player you want to target."] player: Option<String>,
) -> Result<(), SchnoseError> {
	ctx.defer().await?;

	trace!("[/recent] player: {:?}", &player);

	let target = Target::from_input(player, *ctx.author().id.as_u64());
	let player = target.to_player(ctx.database()).await?;

	let records = GlobalAPI::get_recent(&player, Some(10), ctx.gokz_client()).await?;

	let filter = records
		.iter()
		.filter_map(|rec| rec.map_name.as_ref())
		.collect::<Vec<_>>();

	let maps = (*GLOBAL_MAPS)
		.iter()
		.filter(|map| filter.contains(&&map.name))
		.collect::<Vec<_>>();

	let mut embeds = Vec::new();
	let n_pages = records.len();

	for (i, recent) in records.into_iter().enumerate() {
		let place = format!("[#{}]", GlobalAPI::get_place(recent.id, ctx.gokz_client()).await?);

		let (discord_timestamp, footer_msg) =
			chrono::NaiveDateTime::parse_from_str(&recent.created_on, "%Y-%m-%dT%H:%M:%S")
				.map_or_else(
					|_| (String::new(), String::new()),
					|parsed_time| {
						(
							format!("<t:{}:R>", parsed_time.timestamp()),
							format!(
								"Page {} / {} | {} GMT",
								i + 1,
								n_pages,
								parsed_time.format("%d/%m/%Y - %H:%M:%S")
							),
						)
					},
				);

		let mode: Mode = recent.mode.parse()?;

		let teleports = if recent.teleports > 0 {
			format!(" ({} TPs)", recent.teleports)
		} else {
			String::new()
		};

		embeds.push({
			let map = &maps.iter().find(|map| {
				let Some(ref map_name) = recent.map_name else {
					return false;
				};
				&map.name == map_name
			}).expect("Map should be in the cache.");

			let mut embed = CreateEmbed::default();

			embed.color((116, 128, 194))
				.title(format!(
					"[PB] {} on {} (T{})",
					&recent.player_name.unwrap_or_else(|| String::from("unknown")),
					&map.name,
					&map.difficulty
				))
				.url(format!(
					"{}?{}=",
					formatting::map_link(&map.name),
					mode.short().to_lowercase()
				))
				.thumbnail(formatting::map_thumbnail(&map.name))
				.field(
					format!("{} {}", mode.short(), if recent.teleports > 0 { "TP" } else { "PRO" }),
					format!("> {} {}{}\n> {}{}", formatting::format_time(recent.time), place, teleports, discord_timestamp, {
						if recent.replay_id == 0 {
							String::new()
						} else {
							let link = GlobalAPI::get_replay_by_id(recent.replay_id);
							format!("\n> [Watch Replay](http://gokzmaptest.site.nfoservers.com/GlobalReplays/?replay={})\n> [Download Replay]({})",
								recent.replay_id,
								link
							)
						}
					}),
					true
				)
				.footer(|f| f.text(footer_msg).icon_url(crate::ICON));

			embed
		})
	}

	let ctx_id = ctx.id();
	let prev_id = format!("{}_prev", ctx.id());
	let next_id = format!("{}_next", ctx.id());

	// Create initial response
	ctx.send(|reply| {
		reply
			.embed(|e| {
				*e = embeds[0].clone();
				e
			})
			.components(|components| {
				components.create_action_row(|row| {
					row.create_button(|b| b.custom_id(&prev_id).label('◀'))
						.create_button(|b| b.custom_id(&next_id).label('▶'))
				})
			})
	})
	.await?;

	// Loop over incoming interactions of the buttons
	let mut current_page = 0;
	while let Some(press) = CollectComponentInteraction::new(ctx)
		// We only want to handle interactions belonging to the current message
		.filter(move |press| {
			press
				.data
				.custom_id
				.starts_with(&ctx_id.to_string())
		})
		// Listen for 10 minutes
		.timeout(Duration::from_secs(600))
		.await
	{
		if press.data.custom_id != prev_id && press.data.custom_id != next_id {
			// irrelevant interaction
			continue;
		}

		if press.data.custom_id == prev_id {
			if current_page == 0 {
				current_page = embeds.len() - 1;
			} else {
				current_page -= 1;
			}
		} else if press.data.custom_id == next_id {
			current_page += 1;
			if current_page >= embeds.len() {
				current_page = 0;
			}
		}

		// Update message with new page
		press
			.create_interaction_response(ctx, |response| {
				response
					.kind(poise::serenity_prelude::InteractionResponseType::UpdateMessage)
					.interaction_response_data(|data| data.set_embed(embeds[current_page].clone()))
			})
			.await?
	}

	Ok(())
}
