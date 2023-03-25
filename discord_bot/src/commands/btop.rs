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
	poise::serenity_prelude::CreateEmbed,
};

/// Top 100 bonus world record holders.
///
/// This command will fetch the top 100 world record holders for bonuses. You may specify the \
/// following options:
///
/// - `mode`: `KZTimer` / `SimpleKZ` / `Vanilla`
///   - If you don't specify this, the bot will search the database for your UserID. If it can't \
///     find one, or you don't have a mode preference set, the command will fail. To save a mode \
///     preference in the database, see `/mode`.
/// - `runtype`: `TP` / `PRO`
///   - If you don't specify this, the bot will default to `PRO`.
#[tracing::instrument(skip(ctx), fields(user = ctx.author().tag()))]
#[poise::command(slash_command, on_error = "Error::handle_command")]
pub async fn btop(
	ctx: Context<'_>,

	#[description = "KZT/SKZ/VNL"]
	#[rename = "mode"]
	mode_choice: Option<ModeChoice>,

	#[description = "TP/PRO"]
	#[rename = "runtype"]
	runtype_choice: Option<RuntypeChoice>,
) -> Result<()> {
	ctx.defer().await?;

	let db_entry = ctx
		.find_user_by_id(*ctx.author().id.as_u64())
		.await;

	let mode = ModeChoice::parse_input(mode_choice, &db_entry)?;
	let runtype = matches!(runtype_choice, Some(RuntypeChoice::TP));

	let top = global_api::get_wr_top(mode, runtype, 1..101, ctx.gokz_client())
		.await?
		.into_iter()
		.take(100)
		.collect::<Vec<_>>();

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
