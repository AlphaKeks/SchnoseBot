use {
	super::{autocompletion::autocomplete_map, choices::ModeChoice},
	crate::{
		custom_types::Target,
		error::{Error, Result},
		gokz::fmt_time,
		Context, State,
	},
	gokz_rs::{prelude::*, schnose_api},
	log::trace,
};

/// A player's personal best on a bonus course.
///
/// This command will fetch a given player's personal best on a given bonus course. You can \
/// specify the following parameters:
/// - `map_name`: any of [these](https://maps.global-api.com/mapcycles/gokz.txt)
/// - `mode`: filter by mode (KZT/SKZ/VNL)
/// - `player`: `SteamID`, Player Name or @mention
/// - `course`: Which bonus you want to check (i.e. `3` means "bonus 3")
/// If the API has a global replay stored for your run, the bot will attach some links for you to \
/// view and/or download the replay.
#[poise::command(slash_command, on_error = "Error::handle_command")]
pub async fn bpb(
	ctx: Context<'_>,
	#[autocomplete = "autocomplete_map"] map_name: String,
	#[description = "KZT/SKZ/VNL"] mode: Option<ModeChoice>,
	#[description = "The player you want to target."] player: Option<String>,
	#[description = "Course"] course: Option<u8>,
) -> Result<()> {
	trace!("[/bpb ({})]", ctx.author().tag());
	trace!("> `map_name`: {map_name:?}");
	trace!("> `mode`: {mode:?}");
	trace!("> `player`: {player:?}");
	trace!("> `course`: {course:?}");
	ctx.defer().await?;

	let db_entry = ctx
		.find_by_id(*ctx.author().id.as_u64())
		.await;

	let map = ctx.get_map(&MapIdentifier::Name(map_name))?;
	let map_identifier = MapIdentifier::Name(map.name);
	let mode = match mode {
		Some(choice) => Mode::from(choice),
		None => db_entry
			.as_ref()
			.map_err(|_| Error::MissingMode)?
			.mode
			.ok_or(Error::MissingMode)?,
	};
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
	let course = course.unwrap_or(1);

	let tp = schnose_api::get_pb(
		player.clone(),
		map_identifier.clone(),
		course,
		mode,
		true,
		ctx.gokz_client(),
	)
	.await;
	let pro = schnose_api::get_pb(
		player.clone(),
		map_identifier.clone(),
		course,
		mode,
		false,
		ctx.gokz_client(),
	)
	.await;

	let tp_time = if let Ok(tp) = &tp {
		let place = schnose_api::get_place(tp.id, ctx.gokz_client())
			.await
			.map(|place| format!("[#{place}]"))
			.unwrap_or_default();

		format!("{} {}\nby {}", fmt_time(tp.time), place, tp.player.name)
	} else {
		String::from("😔")
	};

	let pro_time = if let Ok(pro) = &pro {
		let place = schnose_api::get_place(pro.id, ctx.gokz_client())
			.await
			.map(|place| format!("[#{place}]"))
			.unwrap_or_default();

		format!("{} {}\nby {}", fmt_time(pro.time), place, pro.player.name)
	} else {
		String::from("😔")
	};

	ctx.send(|replay| {
		replay.embed(|e| {
			e.color(ctx.color())
				.title(format!(
					"[PB] {} on {} B{} (T{})",
					tp.map_or(
						pro.map_or_else(|_| String::from("unknown"), |pro| pro.player.name),
						|tp| tp.player.name
					),
					&map_identifier.to_string(),
					course,
					&map.tier
				))
				.url(format!("{}?{}=&bonus={}", &map.url, mode.short().to_lowercase(), course))
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
