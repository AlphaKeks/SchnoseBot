// TODO: MAKE THIS NOT CRINGE

use {
	super::choices::ModeChoice,
	crate::{
		error::Error, steam_ext::get_steam_avatar, Context, GlobalMapsContainer, State, Target,
		GLOBAL_MAPS,
	},
	gokz_rs::{prelude::*, GlobalAPI, KZGO},
	log::{error, trace},
	num_format::{Locale, ToFormattedString},
	std::collections::HashMap,
};

#[poise::command(slash_command, on_error = "Error::handle_command")]
pub async fn profile(
	ctx: Context<'_>, #[description = "The player you want to target."] player: Option<String>,
	#[description = "KZT/SKZ/VNL"] mode: Option<ModeChoice>,
) -> Result<(), Error> {
	ctx.defer().await?;
	trace!("[/profile] player: `{player:?}`, mode: `{mode:?}`");

	let db_entry = ctx
		.find_by_id(*ctx.author().id.as_u64())
		.await;

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
			let db_entry = db_entry.map_err(|_| Error::NoPlayerInfo)?;

			if let Some(steam_id) = &db_entry.steam_id {
				PlayerIdentifier::SteamID(steam_id.to_owned())
			} else {
				PlayerIdentifier::Name(db_entry.name)
			}
		}
	};

	let tp = GlobalAPI::get_player_records(&player, mode, true, 0, Some(9999), ctx.gokz_client())
		.await
		.unwrap_or_default();

	let pro = GlobalAPI::get_player_records(&player, mode, false, 0, Some(9999), ctx.gokz_client())
		.await
		.unwrap_or_default();

	if tp.is_empty() && pro.is_empty() {
		return Err(Error::NoRecords);
	}

	let x = if tp.len() > pro.len() { tp.len() } else { pro.len() };

	let (mut tp_points, mut pro_points) = (0, 0);
	let mut completion = [(0, 0); 8];
	let (mut tp_records, mut pro_records) = (0, 0);

	let global_maps = GLOBAL_MAPS.try_get()?;

	let mut tier_maps = [
		HashMap::<String, u8>::from_iter(
			global_maps
				.iter()
				.map(|map| (map.name.clone(), map.tier)),
		),
		HashMap::<String, u8>::from_iter(
			global_maps
				.iter()
				.map(|map| (map.name.clone(), map.tier)),
		),
	];

	for i in 0..x {
		if tp.len() > i {
			if let Some(ref map_name) = tp[i].map_name {
				if let Some(tier) = tier_maps[0].remove(map_name) {
					tp_points += tp[i].points;
					completion[7].0 += 1;
					completion[(tier - 1) as usize].0 += 1;

					if tp[i].points == 1000 {
						tp_records += 1;
					}
				}
			}
		}

		if pro.len() > i {
			if let Some(ref map_name) = pro[i].map_name {
				if let Some(tier) = tier_maps[1].remove(map_name) {
					pro_points += pro[i].points;
					completion[7].1 += 1;
					completion[(tier - 1) as usize].1 += 1;

					if pro[i].points == 1000 {
						pro_records += 1;
					}
				}
			}
		}
	}

	let total_points = tp_points + pro_points;

	let rank = Rank::from_points(total_points as u32, mode);

	let possible_maps = KZGO::get_completion_count(mode, ctx.gokz_client()).await?;

	let possible_maps = [
		[
			possible_maps.tp.one, possible_maps.tp.two, possible_maps.tp.three,
			possible_maps.tp.four, possible_maps.tp.five, possible_maps.tp.six,
			possible_maps.tp.seven, possible_maps.tp.total,
		],
		[
			possible_maps.pro.one, possible_maps.pro.two, possible_maps.pro.three,
			possible_maps.pro.four, possible_maps.pro.five, possible_maps.pro.six,
			possible_maps.pro.seven, possible_maps.pro.total,
		],
	];

	let mut completion_percentage = [(0., 0.); 8];

	for i in 0..8 {
		if completion[i].0 > 0 {
			completion_percentage[i].0 =
				(completion[i].0 as f64 / possible_maps[0][i] as f64) * 100.;
		}

		if completion[i].1 > 0 {
			completion_percentage[i].1 =
				(completion[i].1 as f64 / possible_maps[1][i] as f64) * 100.;
		}
	}

	let mut bars = [[""; 7]; 2].map(|a| a.map(String::from));

	for (i, perc) in completion_percentage
		.iter()
		.enumerate()
		.take(7)
	{
		let amount = (perc.0 / 10.) as u32;

		for _ in 0..amount {
			bars[0][i].push('█');
		}

		for _ in 0..(10 - amount) {
			bars[0][i].push('░');
		}

		let amount = (perc.1 / 10.) as u32;

		for _ in 0..amount {
			bars[1][i].push('█');
		}

		for _ in 0..(10 - amount) {
			bars[1][i].push('░');
		}
	}

	let fav_mode = match &player {
		PlayerIdentifier::Name(player_name) => ctx.find_by_name(player_name).await,
		PlayerIdentifier::SteamID(steam_id) => ctx.find_by_steam_id(steam_id).await,
		PlayerIdentifier::SteamID64(steam_id64) => {
			let steam_id = SteamID::from(*steam_id64);
			ctx.find_by_steam_id(&steam_id).await
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
🏆 **World Records: {} (TP) | {} (PRO)**
────────────────────────────
                      TP                                               PRO
        `{}/{} ({:.2}%)`             `{}/{} ({:.2}%)`
T1     ⌠ {} ⌡          ⌠ {} ⌡
T2   ⌠ {} ⌡          ⌠ {} ⌡
T3   ⌠ {} ⌡          ⌠ {} ⌡
T4   ⌠ {} ⌡          ⌠ {} ⌡
T5   ⌠ {} ⌡          ⌠ {} ⌡
T6   ⌠ {} ⌡          ⌠ {} ⌡
T7   ⌠ {} ⌡          ⌠ {} ⌡

Points: **{}**
────────────────────────────
Rank: **{}**
Preferred Mode: {}
        "#,
		tp_records,
		pro_records,
		completion[7].0,
		possible_maps[0][7],
		completion_percentage[7].0,
		completion[7].1,
		possible_maps[1][7],
		completion_percentage[7].1,
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
		total_points.to_formatted_string(&Locale::en),
		rank,
		fav_mode
	);

	let player = GlobalAPI::get_player(&player, ctx.gokz_client()).await?;

	let avatar = get_steam_avatar(&ctx.config().steam_token, &player.steamid64, ctx.gokz_client())
		.await
		.unwrap_or_else(|err| {
			error!("{err:?}");
			ctx.icon().to_owned()
		});

	ctx.send(|reply| {
		reply.embed(|e| {
			e.color(ctx.color())
				.title(format!("{} - {} Profile", &player.name, mode))
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
