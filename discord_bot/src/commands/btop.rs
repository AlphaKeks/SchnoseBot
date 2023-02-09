use {
	super::{
		choices::{ModeChoice, RuntypeChoice},
		pagination::paginate,
	},
	crate::{
		error::Error,
		gokz::{WorldRecordLeaderboard, WorldRecordParams},
		Context, State,
	},
	gokz_rs::{prelude::*, GlobalAPI},
	log::trace,
	poise::serenity_prelude::CreateEmbed,
};

/// Top 100 bonus world record holders.
#[poise::command(slash_command, on_error = "Error::handle_command")]
pub async fn btop(
	ctx: Context<'_>, #[description = "KZT/SKZ/VNL"] mode: Option<ModeChoice>,
	#[description = "TP/PRO"] runtype: Option<RuntypeChoice>,
) -> Result<(), Error> {
	trace!("[/btop ({})]", ctx.author().tag());
	trace!("> `mode`: {mode:?}");
	trace!("> `runtype`: {runtype:?}");
	ctx.defer().await?;

	let db_entry = ctx
		.find_by_id(*ctx.author().id.as_u64())
		.await;

	let mode = match mode {
		Some(choice) => Mode::from(choice),
		None => db_entry
			.map_err(|_| Error::MissingMode)?
			.mode
			.ok_or(Error::MissingMode)?,
	};
	let runtype = matches!(runtype, Some(RuntypeChoice::TP));

	let mut url = format!(
		"/records/top/world_records?mode_ids={}&tickrates=128&has_teleports={}&limit=100",
		mode as u8, runtype
	);

	for stage in 1..=100 {
		url.push_str(&format!("&stages={stage}"));
	}

	let WorldRecordLeaderboard(top) =
		GlobalAPI::get::<WorldRecordLeaderboard, _>(&url, WorldRecordParams, ctx.gokz_client())
			.await?;
	let max_pages = (top.len() as f64 / 12f64).ceil() as u8;

	let mut embeds = Vec::new();
	let mut temp_embed = CreateEmbed::default()
		.color(ctx.color())
		.to_owned();

	let chunk_size = 12;
	let mut place = 1;
	for (page_idx, players) in top.chunks(chunk_size).enumerate() {
		temp_embed
			.title(format!("[BTop 100 {}] World Records", if runtype { "TP" } else { "PRO" },))
			.url(format!("https://kzgo.eu/leaderboards?{}=", mode.short().to_lowercase()))
			.footer(|f| f.text(format!("Mode: {} | Page {} / {}", mode, page_idx + 1, max_pages)));

		for player in players {
			let player_name = &player.player_name;

			temp_embed.field(format!("{player_name} [#{place}]"), player.count, true);
			place += 1;
		}

		embeds.push(temp_embed.clone());
		temp_embed = CreateEmbed::default()
			.color(ctx.color())
			.to_owned();
	}

	paginate(&ctx, embeds).await?;

	Ok(())
}
