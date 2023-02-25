use {
	super::{autocompletion::autocomplete_map, choices::ModeChoice},
	crate::{
		error::{Error, Result},
		gokz::fmt_time,
		Context, State,
	},
	gokz_rs::{prelude::*, schnose_api},
	log::trace,
};

/// World record on a given map.
///
/// This command will fetch the world record on a given map. You can specify the following \
/// parameters:
/// - `map_name`: any of [these](https://maps.global-api.com/mapcycles/gokz.txt)
/// - `mode`: filter by mode (KZT/SKZ/VNL)
/// If the API has a global replay stored for your run, the bot will attach some links for you to \
/// view and/or download the replay.
#[poise::command(slash_command, on_error = "Error::handle_command")]
pub async fn wr(
	ctx: Context<'_>,
	#[autocomplete = "autocomplete_map"] map_name: String,
	#[description = "KZT/SKZ/VNL"] mode: Option<ModeChoice>,
) -> Result<()> {
	trace!("[/wr ({})]", ctx.author().tag());
	trace!("> `map_name`: {map_name:?}");
	trace!("> `mode`: {mode:?}");
	ctx.defer().await?;

	let db_entry = ctx
		.find_by_id(*ctx.author().id.as_u64())
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

	let tp = schnose_api::get_wr(map_identifier.clone(), 0, mode, true, ctx.gokz_client()).await;
	let pro = schnose_api::get_wr(map_identifier.clone(), 0, mode, false, ctx.gokz_client()).await;

	let tp_time = if let Ok(tp) = tp {
		format!("{} ({} TPs)\nby {}", fmt_time(tp.time), tp.teleports, tp.player.name)
	} else {
		String::from("😔")
	};

	let pro_time = if let Ok(pro) = pro {
		format!("{}\nby {}", fmt_time(pro.time), pro.player.name)
	} else {
		String::from("😔")
	};

	ctx.send(|replay| {
		replay.embed(|e| {
			e.color(ctx.color())
				.title(format!("[WR] {} (T{})", &map_identifier.to_string(), &map.tier))
				.url(format!("{}?{}=", &map.url, mode.short().to_lowercase()))
				.thumbnail(&map.thumbnail)
				.field("TP", tp_time, true)
				.field("PRO", pro_time, true)
				.footer(|f| {
					f.text(format!("Mode: {mode}"))
						.icon_url(ctx.icon())
				})
		})
	})
	.await?;

	Ok(())
}
