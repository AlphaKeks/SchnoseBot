use {
	super::{
		autocompletion::autocomplete_map,
		choices::{ModeChoice, RuntypeChoice},
		pagination::paginate,
	},
	crate::{
		error::{Error, Result},
		gokz::fmt_time,
		Context, State,
	},
	gokz_rs::{global_api, MapIdentifier, Mode},
	log::trace,
	poise::serenity_prelude::CreateEmbed,
};

/// Top 100 records on a bonus course.
///
/// This command will fetch the top 100 records on a given bonus course. You can specify the \
/// following parameters:
/// - `map_name`: any of [these](https://maps.global-api.com/mapcycles/gokz.txt)
/// - `mode`: filter by mode (KZT/SKZ/VNL)
/// - `runtype`: TP/PRO
/// - `course`: Which bonus you want to check (i.e. `3` means "bonus 3")
#[poise::command(slash_command, on_error = "Error::handle_command")]
pub async fn bmaptop(
	ctx: Context<'_>,
	#[autocomplete = "autocomplete_map"] map_name: String,
	#[description = "KZT/SKZ/VNL"] mode: Option<ModeChoice>,
	#[description = "TP/PRO"] runtype: Option<RuntypeChoice>,
	#[description = "Course"] course: Option<u8>,
) -> Result<()> {
	trace!("[/bmaptop ({})]", ctx.author().tag());
	trace!("> `map_name`: {map_name:?}");
	trace!("> `mode`: {mode:?}");
	trace!("> `runtype`: {runtype:?}");
	trace!("> `course`: {course:?}");
	ctx.defer().await?;

	let db_entry = ctx
		.find_user_by_id(*ctx.author().id.as_u64())
		.await;

	let map = ctx.get_map(&MapIdentifier::Name(map_name))?;
	let map_identifier = MapIdentifier::Name(map.name);
	let mode = match mode {
		Some(choice) => Mode::from(choice),
		None => db_entry
			.map_err(|_| Error::MissingMode)?
			.mode
			.ok_or(Error::MissingMode)?,
	};
	let runtype = matches!(runtype, Some(RuntypeChoice::TP));
	let course = course.unwrap_or(1);

	let maptop =
		global_api::get_maptop(map_identifier.clone(), mode, runtype, course, ctx.gokz_client())
			.await?;
	let max_pages = (maptop.len() as f64 / 12f64).ceil() as u8;

	let mut embeds = Vec::new();
	let mut temp_embed = CreateEmbed::default()
		.color(ctx.color())
		.to_owned();

	let chunk_size = 12;
	let mut place = 1;
	for (page_idx, records) in maptop.chunks(chunk_size).enumerate() {
		temp_embed
			.title(format!(
				"[Top 100 {}] {} B{} (T{})",
				if runtype { "TP" } else { "PRO" },
				map_identifier,
				course,
				&map.tier
			))
			.url(format!("{}?{}=&bonus={}", &map.url, mode.short().to_lowercase(), course))
			.thumbnail(&map.thumbnail)
			.footer(|f| f.text(format!("Mode: {} | Page {} / {}", mode, page_idx + 1, max_pages)));

		for record in records {
			temp_embed.field(
				format!("{} [#{}]", record.player_name, place),
				fmt_time(record.time),
				true,
			);
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
