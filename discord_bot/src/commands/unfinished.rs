#![allow(unused)] // FIXME

use {
	super::choices::{ModeChoice, RuntypeChoice, TierChoice},
	crate::{
		custom_types::Target,
		error::{Error, Result},
		Context, State,
	},
	gokz_rs::{global_api, schnose_api, Mode, Tier},
	log::trace,
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

	todo!("FIXME");

	// let player = schnose_api::get_player(player, ctx.gokz_client()).await?;
	//
	// ctx.send(|reply| {
	// 	reply.embed(|e| {
	// 		e.color(ctx.color())
	// 			.title(format!(
	// 				"{} - {} {} {}",
	// 				amount,
	// 				mode.short(),
	// 				if runtype { "TP" } else { "PRO" },
	// 				tier.map_or_else(String::new, |tier| format!("[T{}]", tier as u8))
	// 			))
	// 			.description(description)
	// 			.footer(|f| {
	// 				f.text(format!("Player: {}", player.name))
	// 					.icon_url(ctx.icon())
	// 			})
	// 	})
	// })
	// .await?;
	//
	// Ok(())
}
