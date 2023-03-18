use {
	super::{autocompletion::autocomplete_map, choices::ModeChoice},
	crate::{
		error::{Error, Result},
		gokz::{fmt_time, format_replay_links},
		Context, State,
	},
	gokz_rs::{global_api, MapIdentifier},
	log::trace,
};

/// World record on a given bonus course.
///
/// This command will fetch the world record on a given bonus course. You can specify the \
/// following parameters:
/// - `map_name`: any of [these](https://maps.global-api.com/mapcycles/gokz.txt)
/// - `mode`: filter by mode (KZT/SKZ/VNL)
/// - `course`: Which bonus you want to check (i.e. `3` means "bonus 3")
/// If the API has a global replay stored for your run, the bot will attach some links for you to \
/// view and/or download the replay.
#[poise::command(slash_command, on_error = "Error::handle_command")]
pub async fn bwr(
	ctx: Context<'_>,
	#[autocomplete = "autocomplete_map"] map_name: String,
	#[description = "KZT/SKZ/VNL"] mode: Option<ModeChoice>,
	#[description = "Course"] course: Option<u8>,
) -> Result<()> {
	trace!("[/bwr ({})]", ctx.author().tag());
	trace!("> `map_name`: {map_name:?}");
	trace!("> `mode`: {mode:?}");
	trace!("> `course`: {course:?}");
	ctx.defer().await?;

	let db_entry = ctx
		.find_user_by_id(*ctx.author().id.as_u64())
		.await;

	let map = ctx.get_map(&MapIdentifier::Name(map_name))?;
	let map_identifier = MapIdentifier::Name(map.name);
	let mode = ModeChoice::parse_input(mode, &db_entry)?;
	let course = course.unwrap_or(1);

	let tp =
		global_api::get_wr(map_identifier.clone(), mode, true, course, ctx.gokz_client()).await;
	let pro =
		global_api::get_wr(map_identifier.clone(), mode, false, course, ctx.gokz_client()).await;

	let (tp_time, tp_links) = if let Ok(tp) = tp {
		let player_name = format!(
			"[{}](https://kzgo.eu/players/{}?{}=)",
			tp.player_name,
			tp.steam_id,
			mode.short().to_lowercase()
		);

		(
			format!(
				"{} ({} TP{})\n> {}",
				fmt_time(tp.time),
				tp.teleports,
				if tp.teleports > 1 { "s" } else { "" },
				player_name
			),
			Some((tp.replay_download_link(), tp.replay_view_link())),
		)
	} else {
		(String::from("😔"), None)
	};

	let (pro_time, pro_links) = if let Ok(pro) = pro {
		let player_name = format!(
			"[{}](https://kzgo.eu/players/{}?{}=)",
			pro.player_name,
			pro.steam_id,
			mode.short().to_lowercase()
		);

		(
			format!("{}\n> {}", fmt_time(pro.time), player_name),
			Some((pro.replay_download_link(), pro.replay_view_link())),
		)
	} else {
		(String::from("😔"), None)
	};

	ctx.send(|replay| {
		replay.embed(|e| {
			e.color(ctx.color())
				.title(format!(
					"[WR] {} B{} (T{})",
					&map_identifier.to_string(),
					course,
					map.tier as u8
				))
				.url(format!("{}?{}=&bonus={}", &map.url, mode.short().to_lowercase(), course))
				.thumbnail(&map.thumbnail)
				.description(format_replay_links(tp_links, pro_links).unwrap_or_default())
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