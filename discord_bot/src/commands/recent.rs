use {
	super::pagination::paginate,
	crate::{
		error::{Error, Result},
		gokz::fmt_time,
		target::Target,
		Context, State,
	},
	gokz_rs::{global_api, schnose_api, MapIdentifier},
	log::trace,
	poise::serenity_prelude::CreateEmbed,
};

/// Get a player's 10 most recent runs.
///
/// This command will fetch a player's most recent 10 runs (this includes non-pbs and bonus runs). \
/// If there is a global replay available for any of your runs, the bot will attach some links for \
/// watching it online with [GC's replay viewer](https://github.com/GameChaos/GlobalReplays) as \
/// well as downloading the file. You may specify a `player`, which can be any string. The bot \
/// will try its best to interpret it as something useful. If you want to help it with that, \
/// specify one of the following:
///   - a `SteamID`, e.g. `STEAM_1:1:161178172`, `U:1:322356345` or `76561198282622073`
///   - a `Mention`, e.g. `@MyBestFriend`
///   - a player's name, e.g. `AlphaKeks`
///   - If you don't specify this, the bot will search the database for your UserID. If it can't \
///     find one, or you don't have a SteamID set, the command will fail. To save a mode \
///     preference in the database, see `/setsteam`.
#[poise::command(slash_command, on_error = "Error::handle_command")]
pub async fn recent(
	ctx: Context<'_>,

	#[description = "The player you want to target."]
	#[rename = "player"]
	target: Option<String>,
) -> Result<()> {
	trace!("[/recent ({})]", ctx.author().tag());
	trace!("> `target`: {target:?}");
	ctx.defer().await?;

	let db_entry = ctx
		.find_user_by_id(*ctx.author().id.as_u64())
		.await;

	let player_identifier = Target::parse_input(target, db_entry, &ctx).await?;

	let recent_records = schnose_api::get_recent(player_identifier, 10, ctx.gokz_client()).await?;

	let mut embeds = Vec::new();
	let max_records = recent_records.len();

	for (i, record) in recent_records.into_iter().enumerate() {
		let place = global_api::get_place(record.id, ctx.gokz_client())
			.await
			.map(|place| format!("[#{place}]"))?;

		let (map_name, map_tier, map_url, map_thumbnail) = ctx
			.get_map(&MapIdentifier::Name(record.map_name.clone()))
			.map(|map| {
				(
					map.name,
					(map.tier as u8).to_string(),
					format!("{}?{}=", &map.url, record.mode.short().to_lowercase()),
					map.thumbnail,
				)
			})
			.unwrap_or_else(|_| {
				(
					record.map_name.clone(),
					String::from("?"),
					String::new(),
					String::from("https://kzgo.eu/kz_default.png"),
				)
			});

		let n_teleports = if record.teleports > 0 {
			format!(" ({} TP{})", record.teleports, if record.teleports > 1 { "s" } else { "" })
		} else {
			String::new()
		};

		let discord_timestamp = format!("<t:{}:R>", record.created_on.timestamp());
		let player_profile = format!(
			"[Profile](https://kzgo.eu/players/{}?{}=)",
			record.player.steam_id,
			record.mode.short().to_lowercase()
		);

		let mut embed = CreateEmbed::default();
		embed
			.color(ctx.color())
			.title(format!(
				"{} on {}{} (T{})",
				record.player.name,
				&map_name,
				if record.course.stage > 0 {
					format!(" B{}", record.course.stage)
				} else {
					String::new()
				},
				map_tier
			))
			.url(map_url)
			.thumbnail(&map_thumbnail)
			.field(
				format!(
					"{} {}",
					record.mode.short(),
					if record.teleports > 0 { "TP" } else { "PRO" }
				),
				format!(
					"> {} {}{}\n> {}\n> {}",
					fmt_time(record.time),
					place,
					n_teleports,
					discord_timestamp,
					player_profile
				),
				true,
			)
			.footer(|f| {
				f.text(format!(
					"Mode: {} | ID: {} | Page: {} / {}",
					record.mode,
					record.id,
					i + 1,
					max_records
				))
				.icon_url(ctx.icon())
			});

		embeds.push(embed)
	}

	if embeds.len() == 1 {
		ctx.send(|reply| {
			reply.embed(|e| {
				*e = embeds.remove(0);
				e
			})
		})
		.await?;
	} else {
		paginate(&ctx, embeds).await?;
	}

	Ok(())
}
