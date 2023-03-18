use {
	super::{
		choices::{ModeChoice, RuntypeChoice, TierChoice},
		pagination::paginate,
	},
	crate::{
		target::Target,
		error::{Error, Result},
		steam, Context, State,
	},
	gokz_rs::{global_api, kzgo_api, schnose_api, Mode, Tier},
	log::trace,
	poise::serenity_prelude::CreateEmbed,
};

/// Check which maps you still need to finish.
///
/// This command will fetch all maps which you haven't yet completed. You can apply the following \
/// filters to this:
/// - `mode`: filter by mode (KZT/SKZ/VNL)
/// - `runtype`: TP/PRO
/// - `tier`: filter by difficulty
/// - `player`: `SteamID`, Player Name or @mention
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

	let mode = match mode {
		Some(choice) => Mode::from(choice),
		None => db_entry
			.as_ref()
			.map_err(|_| Error::MissingMode)?
			.mode
			.ok_or(Error::MissingMode)?,
	};
	let runtype = matches!(runtype, Some(RuntypeChoice::TP));
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
