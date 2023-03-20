use {
	super::{
		choices::{ModeChoice, RuntypeChoice, TierChoice},
		pagination::paginate,
	},
	crate::{
		error::{Error, Result},
		steam,
		target::Target,
		Context, State,
	},
	gokz_rs::{global_api, kzgo_api, schnose_api, Tier},
	log::trace,
	poise::serenity_prelude::CreateEmbed,
};

/// Check which maps you still need to finish.
///
/// This command will fetch all maps that you haven't finished yet in a particular mode. You may \
/// specify the following parameters:
///
/// - `mode`: `KZTimer` / `SimpleKZ` / `Vanilla`
///   - If you don't specify this, the bot will search the database for your UserID. If it can't \
///     find one, or you don't have a mode preference set, the command will fail. To save a mode \
///     preference in the database, see `/mode`.
/// - `runtype`: `TP` / `PRO`
///   - If you don't specify this, the bot will default to `PRO`.
/// - `tier`: any number from 1-7
///   - If you don't specify this, the bot will fetch maps for all tiers.
/// - `player`: this can be any string. The bot will try its best to interpret it as something \
///   useful. If you want to help it with that, specify one of the following:
///   - a `SteamID`, e.g. `STEAM_1:1:161178172`, `U:1:322356345` or `76561198282622073`
///   - a `Mention`, e.g. `@MyBestFriend`
///   - a player's name, e.g. `AlphaKeks`
///   - If you don't specify this, the bot will search the database for your UserID. If it can't \
///     find one, or you don't have a SteamID set, the command will fail. To save a mode \
///     preference in the database, see `/setsteam`.
#[poise::command(slash_command, on_error = "Error::handle_command")]
pub async fn unfinished(
	ctx: Context<'_>,
	#[description = "KZT/SKZ/VNL"] mode: Option<ModeChoice>,
	#[description = "TP/PRO"] runtype: Option<RuntypeChoice>,
	#[description = "Filter by map difficulty."] tier: Option<TierChoice>,
	#[description = "The player you want to target."] player: Option<String>,
) -> Result<()> {
	trace!("[/unfinished ({})]", ctx.author().tag());
	trace!("> `mode`: {mode:?}");
	trace!("> `runtype`: {runtype:?}");
	trace!("> `tier`: {tier:?}");
	trace!("> `player`: {player:?}");
	ctx.defer().await?;

	let db_entry = ctx
		.find_user_by_id(*ctx.author().id.as_u64())
		.await;

	let mode = ModeChoice::parse_input(mode, &db_entry)?;
	let runtype = matches!(runtype, Some(RuntypeChoice::TP));
	let player_identifier = Target::parse_input(player, db_entry, &ctx).await?;

	let player = schnose_api::get_player(player_identifier.clone(), ctx.gokz_client()).await?;

	let unfinished = global_api::get_unfinished(
		player_identifier,
		mode,
		runtype,
		tier.map(Tier::from),
		ctx.gokz_client(),
	)
	.await?
	.map(|maps| {
		maps.into_iter()
			.map(|map| {
				if tier.is_some() {
					return map.name;
				}
				format!("{} (T{})", map.name, map.difficulty as u8)
			})
			.collect::<Vec<_>>()
	});

	let avatar = if let Ok(user) = kzgo_api::get_avatar(player.steam_id, ctx.gokz_client()).await {
		user.avatar_url
	} else {
		steam::get_steam_avatar(
			&ctx.config().steam_token,
			player.steam_id.as_id64(),
			ctx.gokz_client(),
		)
		.await?
	};

	let mut template = CreateEmbed::default()
		.color(ctx.color())
		.title(format!(
			"{} {} {}",
			mode.short(),
			if runtype { "TP" } else { "PRO" },
			tier.map_or_else(String::new, |tier| format!("[T{}]", tier as u8))
		))
		.url(format!(
			"https://kzgo.eu/players/{}?{}=",
			player.steam_id,
			mode.short().to_lowercase()
		))
		.thumbnail(avatar)
		.description("Congrats! You have no maps left to finish 🥳")
		.footer(|f| {
			f.text(format!("Player: {}", player.name))
				.icon_url(ctx.icon())
		})
		.to_owned();

	match unfinished {
		None => {
			ctx.send(|reply| {
				reply.embed(|e| {
					*e = template;
					e
				})
			})
			.await?;
		}
		Some(maps) if maps.len() <= 10 => {
			let description = maps.join("\n");

			ctx.send(|reply| {
				reply.embed(|e| {
					template.description(description);
					*e = template;
					e
				})
			})
			.await?;
		}
		Some(maps) => {
			let mut embeds = Vec::new();
			let chunk_size = 10;
			let len = maps.len();
			let max_pages = (maps.len() as f64 / chunk_size as f64).ceil() as u8;
			for (page_idx, map_names) in maps.chunks(chunk_size).enumerate() {
				let mut temp = template.clone();
				temp.title(format!(
					"{} maps - {} {} {}",
					len,
					mode.short(),
					if runtype { "TP" } else { "PRO" },
					tier.map_or_else(String::new, |tier| format!("[T{}]", tier as u8))
				))
				.description(map_names.join("\n"))
				.footer(|f| {
					f.text(format!(
						"Player: {} | Page {} / {}",
						&player.name,
						page_idx + 1,
						max_pages
					))
					.icon_url(ctx.icon())
				});

				embeds.push(temp);
			}

			paginate(&ctx, embeds).await?;
		}
	};

	Ok(())
}
