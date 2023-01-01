use {
	super::{MAP_NAMES, autocomplete_map, handle_err, ModeChoice, Target},
	crate::{
		GlobalStateAccess, formatting,
		SchnoseError::{self, *},
		gokz::ExtractRecordInfo,
	},
	log::trace,
	gokz_rs::{prelude::*, GlobalAPI},
};

/// Check a player's personal best on a bonus.
#[poise::command(slash_command, on_error = "handle_err")]
pub async fn bpb(
	ctx: crate::Context<'_>,
	#[autocomplete = "autocomplete_map"] map_name: String,
	#[description = "KZT/SKZ/VNL"] mode: Option<ModeChoice>,
	#[description = "Course"] course: Option<u8>,
	#[description = "The player you want to target."] player: Option<String>,
) -> Result<(), SchnoseError> {
	ctx.defer().await?;

	trace!("[/bpb] map_name: `{}` mode: `{:?}` course = `{:?}`", &map_name, &mode, &course);

	let Some(map_name) = (*MAP_NAMES).iter().find(|name| name.contains(&map_name.to_lowercase())) else {
		return Err(InvalidMapName(map_name));
	};
	let map_name = MapIdentifier::Name(map_name.to_owned());
	let target = Target::from_input(player, *ctx.author().id.as_u64());
	let mode = match mode {
		Some(choice) => choice.into(),
		None => target.get_mode(ctx.database()).await?,
	};
	let course = course.unwrap_or(1);

	let map = GlobalAPI::get_map(&map_name, ctx.gokz_client()).await?;

	let player = target.to_player(ctx.database()).await?;

	let (tp_req, pro_req) = (
		GlobalAPI::get_pb(&player, &map_name, mode, true, course, ctx.gokz_client()).await,
		GlobalAPI::get_pb(&player, &map_name, mode, false, course, ctx.gokz_client()).await,
	);

	let tp = if let Ok(ref tp) = tp_req {
		format!(
			"{}{}",
			formatting::format_time(tp.time),
			match GlobalAPI::get_place(tp.id, ctx.gokz_client()).await {
				Ok(place) => format!(" [#{}]", place),
				_ => String::new(),
			}
		)
	} else {
		String::from("😔")
	};

	let pro = if let Ok(ref pro) = pro_req {
		format!(
			"{}{}",
			formatting::format_time(pro.time),
			match GlobalAPI::get_place(pro.id, ctx.gokz_client()).await {
				Ok(place) => format!(" [#{}]", place),
				_ => String::new(),
			}
		)
	} else {
		String::from("😔")
	};

	let player_name = (&tp_req, &pro_req).get_player_name();
	let (view_link, download_link) = (&tp_req, &pro_req).get_replay_links();

	ctx.send(|reply| {
		reply.embed(|e| {
			e.color((116, 128, 194))
				.title(format!(
					"[BPB {}] {} on {} (T{})",
					course, player_name, &map.name, &map.difficulty
				))
				.url(format!(
					"{}?{}=&bonus={}",
					formatting::map_link(&map.name),
					mode.short().to_lowercase(),
					course
				))
				.thumbnail(formatting::map_thumbnail(&map.name))
				.field("TP", tp, true)
				.field("PRO", pro, true)
				.description(format!("{}\n{}", view_link, download_link))
				.footer(|f| f.text(format!("Mode: {}", mode)).icon_url(crate::ICON))
		})
	})
	.await?;

	Ok(())
}
