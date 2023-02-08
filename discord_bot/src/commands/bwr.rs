use {
	super::{autocompletion::autocomplete_map, choices::ModeChoice},
	crate::{
		error::Error,
		gokz_ext::{fmt_time, GokzRecord},
		Context, GlobalMapsContainer, State, GLOBAL_MAPS,
	},
	gokz_rs::{prelude::*, records::Record, GlobalAPI},
	log::trace,
};

#[poise::command(slash_command, on_error = "Error::handle_command")]
pub async fn bwr(
	ctx: Context<'_>, #[autocomplete = "autocomplete_map"] map_name: String,
	#[description = "KZT/SKZ/VNL"] mode: Option<ModeChoice>,
	#[description = "Course"] course: Option<u8>,
) -> Result<(), Error> {
	ctx.defer().await?;
	trace!("[/bwr] map_name: `{map_name}`, mode: `{mode:?}`, course: `{course:?}`");

	let db_entry = ctx
		.find_by_id(*ctx.author().id.as_u64())
		.await?;

	let map = GLOBAL_MAPS.find(&MapIdentifier::Name(map_name))?;
	let map_identifier = MapIdentifier::Name(map.name);
	let mode = match mode {
		Some(choice) => Mode::from(choice),
		None => db_entry
			.mode
			.ok_or(Error::MissingMode)?,
	};
	let course = course.unwrap_or(1);

	let tp = GlobalAPI::get_wr(&map_identifier, mode, true, course, ctx.gokz_client()).await;
	let pro = GlobalAPI::get_wr(&map_identifier, mode, false, course, ctx.gokz_client()).await;

	let replay_links = Record::formatted_replay_links(tp.as_ref().ok(), pro.as_ref().ok());
	let view_links = Record::formatted_view_links(tp.as_ref().ok(), pro.as_ref().ok());

	let tp_time = if let Ok(tp) = tp {
		format!(
			"{} ({} TPs)\nby {}",
			fmt_time(tp.time),
			tp.teleports,
			tp.player_name
				.unwrap_or_else(|| String::from("unknown"))
		)
	} else {
		String::from("😔")
	};

	let pro_time = if let Ok(pro) = pro {
		format!(
			"{}\nby {}",
			fmt_time(pro.time),
			pro.player_name
				.unwrap_or_else(|| String::from("unknown"))
		)
	} else {
		String::from("😔")
	};

	ctx.send(|replay| {
		replay.embed(|e| {
			e.color(ctx.color())
				.title(format!("[WR] {} B{} (T{})", &map_identifier.to_string(), course, &map.tier))
				.url(format!("{}?{}=&bonus={}", &map.url, mode.short().to_lowercase(), course))
				.thumbnail(&map.thumbnail)
				.description(format!(
					"{}\n{}",
					view_links.unwrap_or_default(),
					replay_links.unwrap_or_default()
				))
				.field("TP", tp_time, true)
				.field("PRO", pro_time, true)
				.footer(|f| {
					f.text(format!("Mode: {mode}"))
						.icon_url(ctx.icon())
				})
		})
	})
	.await?;

	Ok(())
}
