use {
	super::{handle_err, ModeChoice, RuntypeChoice, Target, mode_from_choice},
	crate::{GlobalStateAccess, SchnoseError},
	std::time::Duration,
	log::trace,
	gokz_rs::GlobalAPI,
	poise::serenity_prelude::CreateEmbed,
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorldRecordResponse {
	pub steamid64: String,
	pub steam_id: String,
	pub count: u32,
	pub player_name: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorldRecordResponses(pub Vec<WorldRecordResponse>);

impl gokz_rs::GlobalAPIResponse for WorldRecordResponses {}

impl super::Page for WorldRecordResponse {
	fn to_field(&self, i: usize) -> (String, String, bool) {
		(format!("[#{}] {}", i, self.player_name), self.count.to_string(), true)
	}
}

#[allow(non_snake_case)]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorldRecordParams;

impl gokz_rs::GlobalAPIParams for WorldRecordParams {}

/// Check the top 100 bonus world record holders.
#[poise::command(slash_command, on_error = "handle_err")]
pub async fn btop(
	ctx: crate::Context<'_>,
	#[description = "KZT/SKZ/VNL"] mode: Option<ModeChoice>,
	#[description = "TP/PRO"] runtype: Option<RuntypeChoice>,
) -> Result<(), SchnoseError> {
	ctx.defer().await?;

	trace!("[/btop] mode: `{:?}` runtype: `{:?}`", &mode, &runtype,);

	let mode =
		mode_from_choice(&mode, &Target::None(*ctx.author().id.as_u64()), ctx.database()).await?;
	let runtype = matches!(runtype, Some(RuntypeChoice::TP));

	let url = (2..=100).fold(String::from("/records/top/world_records?stages=1"), |mut link, n| {
		link.push_str(&format!("&stages={}", n));
		link
	});

	let url = format!(
		"{}&mode_ids={}&tickrates=128&has_teleports={}&limit=100",
		url, mode as u8, runtype
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
			.title("Top 100 Bonus Record Holders")
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
