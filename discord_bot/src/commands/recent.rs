use {
	super::pagination::paginate,
	crate::{
		error::Error,
		gokz_ext::{fmt_time, GokzRecord},
		Context, GlobalMapsContainer, State, Target, GLOBAL_MAPS,
	},
	chrono::NaiveDateTime,
	gokz_rs::{prelude::*, GlobalAPI},
	log::trace,
	poise::serenity_prelude::CreateEmbed,
};

#[poise::command(slash_command, on_error = "Error::handle_command")]
pub async fn recent(
	ctx: Context<'_>, #[description = "The player you want to target."] player: Option<String>,
) -> Result<(), Error> {
	ctx.defer().await?;
	trace!("[/recent] player: `{player:?}`");

	let db_entry = ctx
		.find_by_id(*ctx.author().id.as_u64())
		.await;

	let player = match player {
		Some(target) => {
			target
				.parse::<Target>()?
				.into_player(&ctx)
				.await?
		}
		None => {
			let db_entry = db_entry.map_err(|_| Error::NoPlayerInfo)?;

			if let Some(steam_id) = &db_entry.steam_id {
				PlayerIdentifier::SteamID(steam_id.to_owned())
			} else {
				PlayerIdentifier::Name(db_entry.name)
			}
		}
	};

	let recent_records = GlobalAPI::get_recent(&player, Some(10), ctx.gokz_client()).await?;

	let mut embeds = Vec::new();
	let max_records = recent_records.len();

	for (i, record) in recent_records.into_iter().enumerate() {
		let replay_link = record.replay_link();
		let view_link = record.view_link();

		let place = GlobalAPI::get_place(record.id, ctx.gokz_client())
			.await
			.map(|place| format!("[#{place}]"))?;

		let map = GLOBAL_MAPS.find(&MapIdentifier::Name(
			record
				.map_name
				.unwrap_or_else(|| String::from("unknown")),
		))?;

		let mode = record.mode.parse::<Mode>()?;

		let n_teleports = if record.teleports > 0 {
			format!(" ({} TPs)", record.teleports)
		} else {
			String::new()
		};

		let discord_timestamp = NaiveDateTime::parse_from_str(
			&record.created_on, "%Y-%m-%dT%H:%M:%S",
		)
		.map_or_else(|_| String::new(), |parsed_time| format!("<t:{}:R>", parsed_time.timestamp()));

		let mut embed = CreateEmbed::default();
		embed
			.color(ctx.color())
			.title(format!(
				"[PB] {} on {} (T{})",
				record
					.player_name
					.unwrap_or_else(|| String::from("unknown")),
				&map.name,
				&map.tier
			))
			.url(format!("{}?{}=", &map.url, mode.short().to_lowercase()))
			.thumbnail(&map.thumbnail)
			.field(
				format!("{} {}", mode.short(), if record.teleports > 0 { "TP" } else { "PRO" }),
				format!(
					"> {} {}{}\n> {}{}{}",
					fmt_time(record.time),
					place,
					n_teleports,
					discord_timestamp,
					view_link
						.map(|link| format!("\n> [Watch Replay]({link})"))
						.unwrap_or_default(),
					replay_link
						.map(|link| format!("\n> [Download Replay]({link})"))
						.unwrap_or_default()
				),
				true,
			)
			.footer(|f| {
				f.text(format!("Mode: {} | Page: {} / {}", mode, i + 1, max_records))
					.icon_url(ctx.icon())
			});

		embeds.push(embed)
	}

	paginate(&ctx, embeds).await?;

	Ok(())
}
