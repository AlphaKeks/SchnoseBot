use {
	super::{autocomplete_map, ModeChoice},
	crate::{
		error::Error,
		gokz_ext::{fmt_time, GokzRecord},
		Context, GlobalMapsContainer, State, Target, GLOBAL_MAPS,
	},
	gokz_rs::{prelude::*, records::Record, GlobalAPI},
	log::trace,
};

#[poise::command(slash_command, on_error = "Error::handle_command")]
pub async fn pb(
	ctx: Context<'_>, #[autocomplete = "autocomplete_map"] map_name: String,
	#[description = "KZT/SKZ/VNL"] mode: Option<ModeChoice>,
	#[description = "The player you want to target."] player: Option<String>,
) -> Result<(), Error> {
	ctx.defer().await?;
	trace!("[/pb] map_name: `{map_name}`, mode: `{mode:?}`, player: `{player:?}`");

	let db_entry = ctx
		.find_by_id(*ctx.author().id.as_u64())
		.await?;

	let map = GLOBAL_MAPS.find(&MapIdentifier::Name(map_name))?;
	let mode = match mode {
		Some(choice) => Mode::from(choice),
		None => db_entry
			.mode
			.ok_or(Error::MissingMode)?,
	};
	let steam_id = match player {
		Some(target) => {
			target
				.parse::<Target>()?
				.into_steam_id(&ctx)
				.await?
		}
		None => db_entry
			.steam_id
			.ok_or(Error::MissingSteamID { blame_user: true })?,
	};
	let player = PlayerIdentifier::SteamID(steam_id);
	let map_identifier = MapIdentifier::Name(map.name);

	let tp = GlobalAPI::get_pb(&player, &map_identifier, mode, true, 0, ctx.gokz_client()).await;
	let pro = GlobalAPI::get_pb(&player, &map_identifier, mode, false, 0, ctx.gokz_client()).await;

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
					"[PB] {} on {} (T{})",
					player_name(),
					&map_identifier.to_string(),
					&map.tier
				))
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
