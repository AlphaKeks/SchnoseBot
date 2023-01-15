use {
	super::{
		handle_err, ModeChoice, RuntypeChoice, Target,
		btop::{WorldRecordParams, WorldRecordResponses},
	},
	crate::{GlobalStateAccess, SchnoseError},
	std::time::Duration,
	log::trace,
	gokz_rs::GlobalAPI,
	poise::serenity_prelude::CreateEmbed,
};

/// Check the top 100 bonus world record holders.
#[poise::command(slash_command, on_error = "handle_err")]
pub async fn top(
	ctx: crate::Context<'_>,
	#[description = "KZT/SKZ/VNL"] mode: Option<ModeChoice>,
	#[description = "TP/PRO"] runtype: Option<RuntypeChoice>,
) -> Result<(), SchnoseError> {
	ctx.defer().await?;

	trace!("[/btop] mode: `{:?}` runtype: `{:?}`", &mode, &runtype,);

	let mode = match mode {
		Some(mode) => mode.into(),
		None => {
			Target::None(*ctx.author().id.as_u64())
				.get_mode(ctx.database())
				.await?
		},
	};
	let runtype = matches!(runtype, Some(RuntypeChoice::TP));

	let url = format!(
		"/records/top/world_records?stages=0&mode_ids={}&tickrates=128&has_teleports={}&limit=100",
		mode as u8, runtype
	);

	let WorldRecordResponses(leaderboard) =
		GlobalAPI::get::<WorldRecordResponses, WorldRecordParams>(
			&url,
			WorldRecordParams,
			ctx.gokz_client(),
		)
		.await?;

	let get_embed = |i: usize, len: usize| {
		let mut embed = CreateEmbed::default();
		embed
			.color((116, 128, 194))
			.title("Top 100 World Record Holders")
			.url(format!("https://kzgo.eu/leaderboards?{}", mode.short().to_lowercase()))
			.description(format!(
				"Mode: {} | Runtype: {}",
				mode,
				if runtype { "TP" } else { "PRO" }
			))
			.footer(|f| {
				f.text(format!("Page {} / {}", i, len / 12 + 1))
					.icon_url(crate::ICON)
			});
		embed
	};

	super::paginate(leaderboard, get_embed, Duration::from_secs(600), &ctx).await?;

	Ok(())
}
