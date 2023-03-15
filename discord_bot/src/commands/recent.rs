use {
	super::pagination::paginate,
	crate::{
		custom_types::Target,
		error::{Error, Result},
		gokz::fmt_time,
		Context, State,
	},
	gokz_rs::{global_api, schnose_api, MapIdentifier},
	log::trace,
	poise::serenity_prelude::CreateEmbed,
};

/// Get a player's most recent PB. (Main course only)
///
/// Due to limitations with the [GlobalAPI](https://portal.global-api.com/dashboard) this only \
/// works for non-bonus PB runs. It will fetch all of your PBs and then filter them by date to \
/// give you the most recent one. If the API has a global replay stored for your run, the bot will \
/// attach some links for you to view and/or download the replay.
#[poise::command(slash_command, on_error = "Error::handle_command")]
pub async fn recent(
	ctx: Context<'_>,
	#[description = "The player you want to target."] player: Option<String>,
) -> Result<()> {
	trace!("[/recent ({})]", ctx.author().tag());
	trace!("> `player`: {player:?}");
	ctx.defer().await?;

	let player = match player {
		Some(target) => {
			target
				.parse::<Target>()?
				.into_player(&ctx)
				.await?
		}
		None => {
			Target::None(*ctx.author().id.as_u64())
				.into_player(&ctx)
				.await?
		}
	};

	let recent_records = schnose_api::get_recent(player, 10, ctx.gokz_client()).await?;

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
			format!(" ({} TPs)", record.teleports)
		} else {
			String::new()
		};

		let discord_timestamp = format!("<t:{}:R>", record.created_on.timestamp());

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
					"> {} {}{}\n> {}",
					fmt_time(record.time),
					place,
					n_teleports,
					discord_timestamp,
				),
				true,
			)
			.footer(|f| {
				f.text(format!("Mode: {} | Page: {} / {}", record.mode, i + 1, max_records))
					.icon_url(ctx.icon())
			});

		embeds.push(embed)
	}

	paginate(&ctx, embeds).await?;

	Ok(())
}
