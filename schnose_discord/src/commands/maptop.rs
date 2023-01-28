use {
	super::{
		MAP_NAMES, autocomplete_map, handle_err, ModeChoice, RuntypeChoice, Target,
		mode_from_choice,
	},
	crate::{
		GlobalStateAccess, formatting,
		SchnoseError::{self, *},
	},
	std::time::Duration,
	log::trace,
	gokz_rs::{prelude::*, GlobalAPI},
	poise::serenity_prelude::CreateEmbed,
};

/// Check the top 100 records on a map.
#[poise::command(slash_command, on_error = "handle_err")]
pub async fn maptop(
	ctx: crate::Context<'_>,
	#[autocomplete = "autocomplete_map"] map_name: String,
	#[description = "KZT/SKZ/VNL"] mode: Option<ModeChoice>,
	#[description = "TP/PRO"] runtype: Option<RuntypeChoice>,
) -> Result<(), SchnoseError> {
	ctx.defer().await?;

	trace!(
		"[/maptop] map_name: `{}` mode: `{:?}` runtype: `{:?}`",
		&map_name,
		&mode,
		&runtype,
	);

	let Some(map_name) = (*MAP_NAMES).iter().find(|name| name.contains(&map_name.to_lowercase())) else {
		return Err(InvalidMapName(map_name));
	};
	let map_name = MapIdentifier::Name(map_name.to_owned());
	let mode =
		mode_from_choice(&mode, &Target::None(*ctx.author().id.as_u64()), ctx.database()).await?;
	let runtype = matches!(runtype, Some(RuntypeChoice::TP));

	let maptop = GlobalAPI::get_maptop(&map_name, mode, runtype, 0, ctx.gokz_client()).await?;

	let map = GlobalAPI::get_map(&map_name, ctx.gokz_client()).await?;

	let get_embed = |i: usize, len: usize| {
		let mut embed = CreateEmbed::default();
		embed
			.color((116, 128, 194))
			.title(format!(
				"[Top 100 {} {}] {} (T{})",
				mode.short(),
				if runtype { "TP" } else { "PRO" },
				&map.name,
				&map.difficulty
			))
			.url(format!("{}?{}=", formatting::map_link(&map.name), mode.short().to_lowercase()))
			.thumbnail(formatting::map_thumbnail(&map.name))
			.footer(|f| {
				f.text(format!("Page {} / {}", i, (len as f64 / 12.0).floor() as u8))
					.icon_url(crate::ICON)
			});
		embed
	};

	super::paginate(maptop, get_embed, Duration::from_secs(600), &ctx).await?;

	Ok(())
}
