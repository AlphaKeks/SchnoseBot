use {
	super::{autocompletion::autocomplete_map, choices::ModeChoice},
	crate::{
		error::{Error, Result},
		gokz::{fmt_time, format_replay_links},
		target::Target,
		Context, State,
	},
	gokz_rs::{global_api, MapIdentifier},
	log::trace,
};

/// A player's personal best on a bonus course.
///
/// This command will fetch a player's personal best on a particular bonus. You are required to \
/// specify a `map_name` and may also specify the following options:
///
/// - `mode`: `KZTimer` / `SimpleKZ` / `Vanilla`
///   - If you don't specify this, the bot will search the database for your UserID. If it can't \
///     find one, or you don't have a mode preference set, the command will fail. To save a mode \
///     preference in the database, see `/mode`.
/// - `player`: this can be any string. The bot will try its best to interpret it as something \
///   useful. If you want to help it with that, specify one of the following:
///   - a `SteamID`, e.g. `STEAM_1:1:161178172`, `U:1:322356345` or `76561198282622073`
///   - a `Mention`, e.g. `@MyBestFriend`
///   - a player's name, e.g. `AlphaKeks`
///   - If you don't specify this, the bot will search the database for your UserID. If it can't \
///     find one, or you don't have a SteamID set, the command will fail. To save a mode \
///     preference in the database, see `/setsteam`.
/// - `course`: this can be any integer between 1-255.
///   - If you either don't specify this, or put in `0`, the bot will default to `1`.
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
		.find_user_by_id(*ctx.author().id.as_u64())
		.await;

	let map = ctx.get_map(&MapIdentifier::Name(map_name))?;
	let map_identifier = MapIdentifier::Name(map.name);
	let mode = ModeChoice::parse_input(mode, &db_entry)?;
	let player_identifier = Target::parse_input(player, db_entry, &ctx).await?;
	let course = course.unwrap_or(1).max(1);

	let tp = global_api::get_pb(
		player_identifier.clone(),
		map_identifier.clone(),
		mode,
		true,
		course,
		ctx.gokz_client(),
	)
	.await;
	let pro = global_api::get_pb(
		player_identifier.clone(),
		map_identifier.clone(),
		mode,
		false,
		course,
		ctx.gokz_client(),
	)
	.await;

	let mut player_name = String::from("unknown");

	let (tp_time, tp_links) = if let Ok(tp) = &tp {
		player_name = tp.player_name.to_owned();

		let place = global_api::get_place(tp.id, ctx.gokz_client())
			.await
			.map(|place| format!("[#{place}]"))
			.unwrap_or_default();

		(
			format!(
				"{} {} ({} TP{})",
				fmt_time(tp.time),
				place,
				tp.teleports,
				if tp.teleports > 1 { "s" } else { "" }
			),
			Some((tp.replay_download_link(), tp.replay_view_link())),
		)
	} else {
		(String::from("😔"), None)
	};

	let (pro_time, pro_links) = if let Ok(pro) = &pro {
		player_name = pro.player_name.to_owned();

		let place = global_api::get_place(pro.id, ctx.gokz_client())
			.await
			.map(|place| format!("[#{place}]"))
			.unwrap_or_default();

		(
			format!("{} {}", fmt_time(pro.time), place),
			Some((pro.replay_download_link(), pro.replay_view_link())),
		)
	} else {
		(String::from("😔"), None)
	};

	ctx.send(|replay| {
		replay.embed(|e| {
			e.color(ctx.color())
				.title(format!(
					"[PB] {} on {} B{} (T{})",
					player_name, map_identifier, course, map.tier as u8
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
