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

/// World record on a given map.
///
/// This command will fetch the world record on a particular map. You are required to specify a \
/// `map` and may also specify the following options:
///
/// - `mode`: `KZTimer` / `SimpleKZ` / `Vanilla`
///   - If you don't specify this, the bot will search the database for your UserID. If it can't \
///     find one, or you don't have a mode preference set, the command will fail. To save a mode \
///     preference in the database, see `/mode`.
#[poise::command(slash_command, on_error = "Error::handle_command")]
pub async fn wr(
	ctx: Context<'_>,

	#[autocomplete = "autocomplete_map"]
	#[rename = "map"]
	map_choice: String,

	#[description = "KZT/SKZ/VNL"]
	#[rename = "mode"]
	mode_choice: Option<ModeChoice>,
) -> Result<()> {
	trace!("[/wr ({})]", ctx.author().tag());
	trace!("> `map_choice`: {map_choice:?}");
	trace!("> `mode_choice`: {mode_choice:?}");
	ctx.defer().await?;

	let db_entry = ctx
		.find_user_by_id(*ctx.author().id.as_u64())
		.await;

	let map = ctx.get_map(&MapIdentifier::Name(map_choice))?;
	let map_identifier = MapIdentifier::Name(map.name);
	let mode = ModeChoice::parse_input(mode_choice, &db_entry)?;

	let tp = global_api::get_wr(map_identifier.clone(), mode, true, 0, ctx.gokz_client()).await;
	let pro = global_api::get_wr(map_identifier.clone(), mode, false, 0, ctx.gokz_client()).await;

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
			Some((tp.replay_view_link(), tp.replay_download_link())),
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
			Some((pro.replay_view_link(), pro.replay_download_link())),
		)
	} else {
		(String::from("😔"), None)
	};

	ctx.send(|replay| {
		replay.embed(|e| {
			e.color(ctx.color())
				.title(format!("[WR] {} (T{})", &map_identifier, map.tier as u8))
				.url(format!("{}?{}=", &map.url, mode.short().to_lowercase()))
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
