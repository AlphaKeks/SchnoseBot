use {
	super::{autocomplete_map, handle_err, mode_from_choice, ModeChoice, MAP_NAMES},
	crate::{
		commands::Target,
		formatting,
		gokz::ExtractRecordInfo,
		GlobalStateAccess,
		SchnoseError::{self, *},
	},
	gokz_rs::{prelude::*, GlobalAPI},
	log::trace,
};

/// Check the world record on a map.
#[poise::command(slash_command, on_error = "handle_err")]
pub async fn wr(
	ctx: crate::Context<'_>, #[autocomplete = "autocomplete_map"] map_name: String,
	#[description = "KZT/SKZ/VNL"] mode: Option<ModeChoice>,
) -> Result<(), SchnoseError> {
	ctx.defer().await?;

	trace!("[/wr] map_name: `{}` mode: `{:?}`", &map_name, &mode);

	let Some(map_name) = (*MAP_NAMES).iter().find(|name| name.contains(&map_name.to_lowercase())) else {
		return Err(InvalidMapName(map_name));
	};
	let map_name = MapIdentifier::Name(map_name.to_owned());
	let mode =
		mode_from_choice(&mode, &Target::None(*ctx.author().id.as_u64()), ctx.database()).await?;

	let map = GlobalAPI::get_map(&map_name, ctx.gokz_client()).await?;

	let (tp_req, pro_req) = (
		GlobalAPI::get_wr(&map_name, mode, true, 0, ctx.gokz_client()).await,
		GlobalAPI::get_wr(&map_name, mode, false, 0, ctx.gokz_client()).await,
	);

	let (view_link, download_link) = (&tp_req, &pro_req).get_replay_links();

	let tp = if let Ok(tp) = tp_req {
		format!(
			"{} ({} TPs)\nby {}",
			formatting::format_time(tp.time),
			tp.teleports,
			tp.player_name
				.unwrap_or_else(|| String::from("unknown"))
		)
	} else {
		String::from("😔")
	};

	let pro = if let Ok(pro) = pro_req {
		format!(
			"{}\nby {}",
			formatting::format_time(pro.time),
			pro.player_name
				.unwrap_or_else(|| String::from("unknown"))
		)
	} else {
		String::from("😔")
	};

	ctx.send(|reply| {
		reply.embed(|e| {
			e.color((116, 128, 194))
				.title(format!("[WR] {} (T{})", &map.name, &map.difficulty))
				.url(format!(
					"{}?{}=",
					formatting::map_link(&map.name),
					mode.short().to_lowercase()
				))
				.thumbnail(formatting::map_thumbnail(&map.name))
				.field("TP", tp, true)
				.field("PRO", pro, true)
				.description(format!("{view_link}\n{download_link}"))
				.footer(|f| {
					f.text(format!("Mode: {mode}"))
						.icon_url(crate::ICON)
				})
		})
	})
	.await?;

	Ok(())
}
