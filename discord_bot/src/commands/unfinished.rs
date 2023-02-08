use {
	super::choices::{ModeChoice, RuntypeChoice, TierChoice},
	crate::{error::Error, Context, State, Target},
	gokz_rs::{prelude::*, GlobalAPI},
	log::trace,
};

#[poise::command(slash_command, on_error = "Error::handle_command")]
pub async fn unfinished(
	ctx: Context<'_>, #[description = "KZT/SKZ/VNL"] mode: Option<ModeChoice>,
	#[description = "TP/PRO"] runtype: Option<RuntypeChoice>,
	#[description = "Filter by map difficulty."] tier: Option<TierChoice>,
	#[description = "The player you want to target."] player: Option<String>,
) -> Result<(), Error> {
	ctx.defer().await?;
	trace!("[/unfinished] mode: `{mode:?}`, runtype: `{runtype:?}`, tier: `{tier:?}`, player: `{player:?}`");

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
	let runtype = matches!(runtype, Some(RuntypeChoice::TP));
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

	let (description, amount) = gokz_rs::extra::get_unfinished(
		&player,
		mode,
		runtype,
		tier.map(Tier::from),
		ctx.gokz_client(),
	)
	.await
	.map(|map_names| {
		let description = if map_names.len() <= 10 {
			map_names.join("\n")
		} else {
			format!("{}\n...{} more", map_names[0..10].join("\n"), map_names.len() - 10)
		};

		// I love and hate this at the same time.
		let amount = format!(
			"{} uncompleted map{}",
			map_names.len(),
			if map_names.len() == 1 { "" } else { "s" }
		);

		(description, amount)
	})?;

	let player = GlobalAPI::get_player(&player, ctx.gokz_client()).await?;

	ctx.send(|reply| {
		reply.embed(|e| {
			e.color(ctx.color())
				.title(format!(
					"{} - {} {} {}",
					amount,
					mode.short(),
					if runtype { "TP" } else { "PRO" },
					tier.map_or_else(String::new, |tier| format!("[T{}]", tier as u8))
				))
				.description(if description.is_empty() {
					String::from("You have no maps left to complete! Congrats! 🥳")
				} else {
					description
				})
				.footer(|f| {
					f.text(format!("Player: {}", player.name))
						.icon_url(ctx.icon())
				})
		})
	})
	.await?;

	Ok(())
}
