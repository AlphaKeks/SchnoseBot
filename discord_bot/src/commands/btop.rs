use {
	super::{
		choices::{ModeChoice, RuntypeChoice},
		pagination::paginate,
	},
	crate::{
		error::{Error, Result},
		Context, State,
	},
	gokz_rs::global_api,
	log::trace,
	poise::serenity_prelude::CreateEmbed,
};

/// Top 100 bonus world record holders.
///
/// This command will fetch the top 100 bonus world records holders for TP or PRO. You can specify \
/// the following parameters:
/// - `mode`: filter by mode (KZT/SKZ/VNL)
/// - `runtype`: TP/PRO
#[poise::command(slash_command, on_error = "Error::handle_command")]
pub async fn btop(
	ctx: Context<'_>,
	#[description = "KZT/SKZ/VNL"] mode: Option<ModeChoice>,
	#[description = "TP/PRO"] runtype: Option<RuntypeChoice>,
) -> Result<()> {
	trace!("[/btop ({})]", ctx.author().tag());
	trace!("> `mode`: {mode:?}");
	trace!("> `runtype`: {runtype:?}");
	ctx.defer().await?;

	let db_entry = ctx
		.find_user_by_id(*ctx.author().id.as_u64())
		.await;

	let mode = ModeChoice::parse_input(mode, &db_entry)?;
	let runtype = matches!(runtype, Some(RuntypeChoice::TP));

	let top = global_api::get_wr_top(mode, runtype, 1..101, ctx.gokz_client()).await?;
	let max_pages = (top.len() as f64 / 12f64).ceil() as u8;

	let mut embeds = Vec::new();
	let mut temp_embed = CreateEmbed::default()
		.color(ctx.color())
		.to_owned();

	let chunk_size = 12;
	let mut place = 1;
	for (page_idx, players) in top.chunks(chunk_size).enumerate() {
		temp_embed
			.title(format!(
				"[Top 100 {}] Bonus World Record Holders",
				if runtype { "TP" } else { "PRO" },
			))
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