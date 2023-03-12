//! FIXME

use {
	super::choices::ModeChoice,
	crate::{
		custom_types::Target,
		error::{Error, Result},
		steam::get_steam_avatar,
		Context, State,
	},
	gokz_rs::{global_api, kzgo_api, schnose_api, Mode, PlayerIdentifier, Rank},
	log::trace,
	num_format::{Locale, ToFormattedString},
	std::collections::{hash_map::RandomState, HashMap},
};

/// Similar to how a player profile is displayed on KZ:GO. (I tried my best...)
///
/// This command will fetch a bunch of information about you and is meant to somewhat replicate \
/// the profile view of [KZ:GO](https://kzgo.eu). It will show some bars representing your \
/// completion % for each tier as well as your amount of world records, total points, rank and \
/// preferred mode.
#[poise::command(slash_command, on_error = "Error::handle_command")]
pub async fn profile(
	ctx: Context<'_>,
	#[description = "The player you want to look up."] player: Option<String>,
	#[description = "KZT/SKZ/VNL"] mode: Option<ModeChoice>,
) -> Result<()> {
	trace!("[/profile ({})]", ctx.author().tag());
	trace!("> `player`: {player:?}");
	trace!("> `mode`: {mode:?}");
	ctx.defer().await?;

	let db_entry = ctx
		.find_user_by_id(*ctx.author().id.as_u64())
		.await;

	let mode = match mode {
		Some(choice) => Mode::from(choice),
		None => db_entry
			.as_ref()
			.map_err(|_| Error::MissingMode)?
			.mode
			.ok_or(Error::MissingMode)?,
	};
	let player_identifier = match player {
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

	let player = schnose_api::get_player(player_identifier.clone(), ctx.gokz_client()).await?;

	let tp = global_api::get_player_records(
		player_identifier.clone(),
		mode,
		true,
		0,
		9999,
		ctx.gokz_client(),
	)
	.await
	.unwrap_or_default();

	let pro = global_api::get_player_records(
		player_identifier.clone(),
		mode,
		false,
		0,
		9999,
		ctx.gokz_client(),
	)
	.await
	.unwrap_or_default();

	if tp.is_empty() && pro.is_empty() {
		return Err(Error::NoRecords);
	}

	let mut tp_points = 0;
	let mut pro_points = 0;
	let mut completion_count = [(0, 0); 8];
	let mut tp_wrs = 0;
	let mut pro_wrs = 0;
	let (total_tp_records, total_pro_records) = match mode {
		Mode::KZTimer => (player.records.kzt.tp, player.records.kzt.pro),
		Mode::SimpleKZ => (player.records.skz.tp, player.records.skz.pro),
		Mode::Vanilla => (player.records.vnl.tp, player.records.vnl.pro),
	};
	let mut tp_maps: HashMap<u16, u8, RandomState> = HashMap::from_iter(
		ctx.global_maps()
			.iter()
			.map(|map| (map.id, map.tier as u8)),
	);
	let mut pro_maps = tp_maps.clone();

	let len = tp.len().max(pro.len());
	for i in 0..len {
		if tp.len() > i {
			let map_id = tp[i].map_id;
			if let Some(tier) = tp_maps.remove(&map_id) {
				let points = tp[i].points;

				tp_points += points;
				completion_count[0].0 += 1;
				completion_count[tier as usize].0 += 1;

				if points == 1000 {
					tp_wrs += 1;
				}
			}
		}

		if pro.len() > i {
			let map_id = pro[i].map_id;
			if let Some(tier) = pro_maps.remove(&map_id) {
				let points = pro[i].points;

				pro_points += points;
				completion_count[0].1 += 1;
				completion_count[tier as usize].1 += 1;

				if points == 1000 {
					pro_wrs += 1;
				}
			}
		}
	}

	let total_points = tp_points + pro_points;
	let rank = Rank::from_points(total_points, mode);

	let completion_stats = kzgo_api::get_completions(mode, ctx.gokz_client()).await?;
	let mut completion_percentages = [(0f64, 0f64); 8];

	for i in 0..8 {
		// player has completed maps in this tier in tp
		if completion_count[i].0 > 0 {
			let count = completion_count[i].0;
			let max_count = completion_stats.tp[i];
			completion_percentages[i].0 = (count as f64 / max_count as f64) * 100f64;
		}

		// player has completed maps in this tier in pro
		if completion_count[i].1 > 0 {
			let count = completion_count[i].1;
			let max_count = completion_stats.pro[i];
			completion_percentages[i].1 = (count as f64 / max_count as f64) * 100f64;
		}
	}

	let mut bars = [[""; 7]; 2].map(|bars| bars.map(String::from));

	for (i, percentage) in completion_percentages
		.iter()
		.skip(1)
		.enumerate()
	{
		let amount = (percentage.0 / 10f64) as u32;

		for _ in 0..amount {
			bars[0][i].push('█');
		}

		for _ in 0..(10 - amount) {
			bars[0][i].push('░');
		}

		let amount = (percentage.1 / 10f64) as u32;

		for _ in 0..amount {
			bars[1][i].push('█');
		}

		for _ in 0..(10 - amount) {
			bars[1][i].push('░');
		}
	}

	let fav_mode = match &player_identifier {
		PlayerIdentifier::Name(player_name) => ctx.find_user_by_name(player_name).await,
		PlayerIdentifier::SteamID(steam_id) => {
			ctx.find_user_by_steam_id(steam_id)
				.await
		}
	}
	.map_or_else(
		|_| String::from("unknown"),
		|user| {
			user.mode
				.map_or_else(|| String::from("unknown"), |mode| mode.to_string())
		},
	);

	let description = format!(
		r#"
🏆 **TP**: {}
🏆 **PRO**: {}
──────────────────────────────────────────
```
        [TP]                 [PRO]
  {}/{} ({:.2}%)      {}/{} ({:.2}%)
T1 ⌠ {} ⌡        ⌠ {} ⌡
T2 ⌠ {} ⌡        ⌠ {} ⌡
T3 ⌠ {} ⌡        ⌠ {} ⌡
T4 ⌠ {} ⌡        ⌠ {} ⌡
T5 ⌠ {} ⌡        ⌠ {} ⌡
T6 ⌠ {} ⌡        ⌠ {} ⌡
T7 ⌠ {} ⌡        ⌠ {} ⌡

Total TP runs:  {}
Total PRO runs: {}
```──────────────────────────────────────────
Points: **{} ({})**
Preferred Mode: {}
		"#,
		tp_wrs,
		pro_wrs,
		completion_count[0].0,
		completion_stats.tp[0],
		completion_percentages[0].0,
		completion_count[0].1,
		completion_stats.pro[0],
		completion_percentages[1].0,
		bars[0][0],
		bars[1][0],
		bars[0][1],
		bars[1][1],
		bars[0][2],
		bars[1][2],
		bars[0][3],
		bars[1][3],
		bars[0][4],
		bars[1][4],
		bars[0][5],
		bars[1][5],
		bars[0][6],
		bars[1][6],
		total_tp_records,
		total_pro_records,
		total_points.to_formatted_string(&Locale::en),
		rank,
		fav_mode
	);

	let avatar = if let Ok(avatar) =
		get_steam_avatar(&ctx.config().steam_token, player.steam_id.as_id64(), ctx.gokz_client())
			.await
	{
		avatar
	} else {
		kzgo_api::get_avatar(player.steam_id, ctx.gokz_client())
			.await?
			.avatar_url
	};

	ctx.send(|reply| {
		reply.embed(|e| {
			e.color(ctx.color())
				.title(format!("[{}] {}", mode.short(), &player.name))
				.url(format!(
					"https://kzgo.eu/players/{}?{}=",
					&player.steam_id,
					mode.short().to_lowercase()
				))
				.thumbnail(avatar)
				.description(description)
				.footer(|f| {
					f.text(format!("SteamID: {}", &player.steam_id))
						.icon_url(ctx.icon())
				})
		})
	})
	.await?;

	Ok(())
}
