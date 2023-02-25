use {
	super::pagination::paginate,
	crate::{
		custom_types::Target,
		error::{Error, Result},
		gokz::fmt_time,
		Context, State,
	},
	chrono::NaiveDateTime,
	gokz_rs::{prelude::*, schnose_api},
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
	ctx: Context<'_>, #[description = "The player you want to target."] player: Option<String>,
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

	let recent_records = schnose_api::get_records(player, 10, ctx.gokz_client()).await?;

	let mut embeds = Vec::new();
	let max_records = recent_records.len();

	for (i, record) in recent_records.into_iter().enumerate() {
		let place = schnose_api::get_place(record.id, ctx.gokz_client())
			.await
			.map(|place| format!("[#{place}]"))
			.unwrap_or_default();

		let map = ctx.get_map(&MapIdentifier::Name(record.map_name))?;

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
			.title(format!("[PB] {} on {} (T{})", record.player.name, &map.name, &map.tier))
			.url(format!("{}?{}=", &map.url, mode.short().to_lowercase()))
			.thumbnail(&map.thumbnail)
			.field(
				format!("{} {}", mode.short(), if record.teleports > 0 { "TP" } else { "PRO" }),
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
				f.text(format!("Mode: {} | Page: {} / {}", mode, i + 1, max_records))
					.icon_url(ctx.icon())
			});

		embeds.push(embed)
	}

	paginate(&ctx, embeds).await?;

	Ok(())
}
