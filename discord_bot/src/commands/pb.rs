use {
	super::{autocompletion::autocomplete_map, choices::ModeChoice},
	crate::{
		target::Target,
		error::{Error, Result},
		gokz::{fmt_time, format_replay_links},
		Context, State,
	},
	gokz_rs::{global_api, MapIdentifier, Mode},
	log::trace,
};

/// A player's personal best on a map.
///
/// This command will fetch a given player's personal best on a given map. You can specify the \
/// following parameters:
/// - `map_name`: any of [these](https://maps.global-api.com/mapcycles/gokz.txt)
/// - `mode`: filter by mode (KZT/SKZ/VNL)
/// - `player`: `SteamID`, Player Name or @mention
/// If the API has a global replay stored for your run, the bot will attach some links for you to \
/// view and/or download the replay.
#[poise::command(slash_command, on_error = "Error::handle_command")]
pub async fn pb(
	ctx: Context<'_>,
	#[autocomplete = "autocomplete_map"] map_name: String,
	#[description = "KZT/SKZ/VNL"] mode: Option<ModeChoice>,
	#[description = "The player you want to target."] player: Option<String>,
) -> Result<()> {
	trace!("[/pb ({})]", ctx.author().tag());
	trace!("> `map_name`: {map_name:?}");
	trace!("> `mode`: {mode:?}");
	trace!("> `player`: {player:?}");
	ctx.defer().await?;

	let db_entry = ctx
		.find_user_by_id(*ctx.author().id.as_u64())
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

	let tp = global_api::get_pb(
		player.clone(),
		map_identifier.clone(),
		mode,
		true,
		0,
		ctx.gokz_client(),
	)
	.await;
	let pro = global_api::get_pb(
		player.clone(),
		map_identifier.clone(),
		mode,
		false,
		0,
		ctx.gokz_client(),
	)
	.await;

	let mut player_name = String::from("unknown");

	let (tp_time, tp_links) = if let Ok(tp) = &tp {
		player_name = tp.player_name.clone();

		let place = global_api::get_place(tp.id, ctx.gokz_client())
			.await
			.map(|place| format!("[#{place}]"))
			.unwrap_or_default();

		let player_name = format!(
			"[{}](https://kzgo.eu/players/{}?{}=)",
			tp.player_name,
			tp.steam_id,
			mode.short().to_lowercase()
		);

		(
			format!("{} {}\nby {}", fmt_time(tp.time), place, player_name),
			Some((tp.replay_view_link(), tp.replay_download_link())),
		)
	} else {
		(String::from("😔"), None)
	};

	let (pro_time, pro_links) = if let Ok(pro) = &pro {
		player_name = pro.player_name.clone();

		let place = global_api::get_place(pro.id, ctx.gokz_client())
			.await
			.map(|place| format!("[#{place}]"))
			.unwrap_or_default();

		let player_name = format!(
			"[{}](https://kzgo.eu/players/{}?{}=)",
			pro.player_name,
			pro.steam_id,
			mode.short().to_lowercase()
		);

		(
			format!("{} {}\nby {}", fmt_time(pro.time), place, player_name),
			Some((pro.replay_view_link(), pro.replay_download_link())),
		)
	} else {
		(String::from("😔"), None)
	};

	ctx.send(|replay| {
		replay.embed(|e| {
			e.color(ctx.color())
				.title(format!(
					"[PB] {} on {} (T{})",
					player_name,
					&map_identifier.to_string(),
					map.tier as u8
				))
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
