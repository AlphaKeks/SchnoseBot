use {
	super::{autocompletion::autocomplete_map, choices::ModeChoice},
	crate::{
		error::Error,
		gokz_ext::{fmt_time, GokzRecord},
		Context, State, Target,
	},
	gokz_rs::{prelude::*, records::Record, GlobalAPI},
	log::trace,
};

#[poise::command(slash_command, on_error = "Error::handle_command")]
pub async fn bpb(
	ctx: Context<'_>, #[autocomplete = "autocomplete_map"] map_name: String,
	#[description = "KZT/SKZ/VNL"] mode: Option<ModeChoice>,
	#[description = "The player you want to target."] player: Option<String>,
	#[description = "Course"] course: Option<u8>,
) -> Result<(), Error> {
	ctx.defer().await?;
	trace!("[/bpb] map_name: `{map_name}`, mode: `{mode:?}`, player: `{player:?}`");

	let db_entry = ctx
		.find_by_id(*ctx.author().id.as_u64())
		.await;

	let map = ctx.get_map(&MapIdentifier::Name(map_name))?;
	let map_identifier = MapIdentifier::Name(map.name);
	let mode = match mode {
		Some(choice) => Mode::from(choice),
		None => db_entry
			.as_ref()
			.map_err(|_| Error::MissingMode)?
			.mode
			.ok_or(Error::MissingMode)?,
	};
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
	let course = course.unwrap_or(1);

	let tp =
		GlobalAPI::get_pb(&player, &map_identifier, mode, true, course, ctx.gokz_client()).await;
	let pro =
		GlobalAPI::get_pb(&player, &map_identifier, mode, false, course, ctx.gokz_client()).await;

	let replay_links = Record::formatted_replay_links(tp.as_ref().ok(), pro.as_ref().ok());
	let view_links = Record::formatted_view_links(tp.as_ref().ok(), pro.as_ref().ok());

	let player_name = || {
		if let Ok(tp) = &tp {
			if let Some(name) = &tp.player_name {
				return name.to_owned();
			}
		}

		if let Ok(pro) = &pro {
			if let Some(name) = &pro.player_name {
				return name.to_owned();
			}
		}

		String::from("unknown")
	};

	let tp_time = if let Ok(tp) = &tp {
		let place = GlobalAPI::get_place(tp.id, ctx.gokz_client())
			.await
			.map(|place| format!("[#{place}]"))
			.unwrap_or_default();

		format!("{} {}\nby {}", fmt_time(tp.time), place, player_name())
	} else {
		String::from("😔")
	};

	let pro_time = if let Ok(pro) = &pro {
		let place = GlobalAPI::get_place(pro.id, ctx.gokz_client())
			.await
			.map(|place| format!("[#{place}]"))
			.unwrap_or_default();

		format!("{} {}\nby {}", fmt_time(pro.time), place, player_name())
	} else {
		String::from("😔")
	};

	ctx.send(|replay| {
		replay.embed(|e| {
			e.color(ctx.color())
				.title(format!(
					"[PB] {} on {} B{} (T{})",
					player_name(),
					&map_identifier.to_string(),
					course,
					&map.tier
				))
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
