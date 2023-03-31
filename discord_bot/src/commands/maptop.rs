use {
	super::{
		autocompletion::autocomplete_map,
		choices::{ModeChoice, RuntypeChoice},
		pagination::paginate,
	},
	crate::{
		error::{Error, Result},
		Context, State,
	},
	gokz_rs::{global_api, MapIdentifier},
	poise::serenity_prelude::CreateEmbed,
	schnosebot::formatting::fmt_time,
};

/// Top 100 records on a map.
///
/// This command will fetch the top 100 (or less, if there are less than 100 completions) records \
/// on a particular map. You are required to specify a `map` and may also specify the \
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
pub async fn maptop(
	ctx: Context<'_>,

	#[autocomplete = "autocomplete_map"]
	#[rename = "map"]
	map_choice: String,

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

	let map = ctx.get_map(map_choice)?;
	let map_identifier = MapIdentifier::Name(map.name);
	let mode = ModeChoice::parse_input(mode_choice, &db_entry)?;
	let runtype = matches!(runtype_choice, Some(RuntypeChoice::TP));

	let maptop =
		global_api::get_maptop(map_identifier.clone(), mode, runtype, 0, ctx.gokz_client()).await?;

	let mut embeds = Vec::new();
	let mut temp_embed = CreateEmbed::default()
		.color(ctx.color())
		.to_owned();

	let chunk_size = 12;
	let mut place = 1;
	let max_pages = (maptop.len() as f64 / chunk_size as f64).ceil() as u8;
	for (page_idx, records) in maptop.chunks(chunk_size).enumerate() {
		temp_embed
			.title(format!(
				"[Top 100 {}] {} (T{})",
				if runtype { "TP" } else { "PRO" },
				map_identifier,
				map.tier as u8
			))
			.url(format!("{}?{}=", &map.url, mode.short().to_lowercase()))
			.thumbnail(&map.thumbnail)
			.footer(|f| f.text(format!("Mode: {} | Page {} / {}", mode, page_idx + 1, max_pages)));

		for record in records {
			temp_embed.field(
				format!("{} [#{}]", record.player_name, place),
				format!(
					"{}{}",
					fmt_time(record.time),
					if record.teleports > 0 {
						format!(
							" ({} TP{})",
							record.teleports,
							if record.teleports > 1 { "s" } else { "" }
						)
					} else {
						String::new()
					}
				),
				true,
			);
			place += 1;
		}

		embeds.push(temp_embed.clone());
		temp_embed = CreateEmbed::default()
			.color(ctx.color())
			.to_owned();
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
