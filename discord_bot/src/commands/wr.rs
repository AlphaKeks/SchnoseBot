use {
	super::{autocompletion::autocomplete_map, choices::ModeChoice},
	crate::{
		error::Error,
		gokz::{fmt_time, GokzRecord},
		Context, State,
	},
	gokz_rs::{prelude::*, records::Record, GlobalAPI},
	log::trace,
};

/// Check a world record.
#[poise::command(slash_command, on_error = "Error::handle_command")]
pub async fn wr(
	ctx: Context<'_>, #[autocomplete = "autocomplete_map"] map_name: String,
	#[description = "KZT/SKZ/VNL"] mode: Option<ModeChoice>,
) -> Result<(), Error> {
	trace!("[/wr ({})]", ctx.author().tag());
	trace!("> `map_name`: {map_name:?}");
	trace!("> `mode`: {mode:?}");
	ctx.defer().await?;

	let db_entry = ctx
		.find_by_id(*ctx.author().id.as_u64())
		.await;

	let map = ctx.get_map(&MapIdentifier::Name(map_name))?;
	let map_identifier = MapIdentifier::Name(map.name);
	let mode = match mode {
		Some(choice) => Mode::from(choice),
		None => db_entry
			.map_err(|_| Error::MissingMode)?
			.mode
			.ok_or(Error::MissingMode)?,
	};

	let tp = GlobalAPI::get_wr(&map_identifier, mode, true, 0, ctx.gokz_client()).await;
	let pro = GlobalAPI::get_wr(&map_identifier, mode, false, 0, ctx.gokz_client()).await;

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
				.title(format!("[WR] {} (T{})", &map_identifier.to_string(), &map.tier))
				.url(format!("{}?{}=", &map.url, mode.short().to_lowercase()))
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
