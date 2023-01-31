use {
	super::{handle_err, mode_from_choice, ModeChoice, RuntypeChoice, Target, TierChoice},
	crate::{GlobalStateAccess, SchnoseError},
	gokz_rs::{prelude::*, GlobalAPI},
	log::trace,
};

/// Check which maps a player still has to complete.
#[poise::command(slash_command, on_error = "handle_err")]
pub async fn unfinished(
	ctx: crate::Context<'_>, #[description = "KZT/SKZ/VNL"] mode: Option<ModeChoice>,
	#[description = "TP/PRO"] runtype: Option<RuntypeChoice>,
	#[description = "Filter by map difficulty."] tier: Option<TierChoice>,
	#[description = "The player you want to target."] player: Option<String>,
) -> Result<(), SchnoseError> {
	ctx.defer().await?;

	trace!(
		"[/unfinished] mode: `{:?}` runtype: `{:?}` tier: `{:?}` player: `{:?}`",
		&mode,
		&runtype,
		&tier,
		&player
	);

	let target = Target::from_input(player, *ctx.author().id.as_u64());
	let mode = mode_from_choice(&mode, &target, ctx.database()).await?;
	let player = target.to_player(ctx.database()).await?;
	let runtype = matches!(runtype, Some(RuntypeChoice::TP));
	let tier = tier.map(Tier::from);

	let (description, amount) = {
		let unfinished =
			gokz_rs::extra::get_unfinished(&player, mode, runtype, tier, ctx.gokz_client()).await?;
		let description = if unfinished.len() <= 10 {
			unfinished.join("\n")
		} else {
			format!("{}\n...{} more", unfinished[0..10].join("\n"), unfinished.len() - 10)
		};

		let amount = format!(
			"{} uncompleted map{}",
			unfinished.len(),
			if unfinished.len() == 1 { "" } else { "s" }
		);

		(description, amount)
	};

	let player = GlobalAPI::get_player(&player, ctx.gokz_client()).await?;

	ctx.send(|reply| {
		reply.embed(|e| {
			e.color((116, 128, 194))
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
						.icon_url(crate::ICON)
				})
		})
	})
	.await?;

	Ok(())
}
