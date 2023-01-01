use {
	super::{GLOBAL_MAPS, handle_err, Target},
	crate::{GlobalStateAccess, formatting, SchnoseError},
	log::trace,
	gokz_rs::{prelude::*, GlobalAPI},
};

/// Check a player's most recently set personal best.
#[poise::command(slash_command, on_error = "handle_err")]
pub async fn recent(
	ctx: crate::Context<'_>,
	#[description = "The player you want to target."] player: Option<String>,
) -> Result<(), SchnoseError> {
	ctx.defer().await?;

	trace!("[/recent] player: {:?}", &player);

	let target = Target::from_input(player, *ctx.author().id.as_u64());
	let player = target.to_player(ctx.database()).await?;

	let recent = GlobalAPI::get_recent(&player, ctx.gokz_client()).await?;

	let map = (*GLOBAL_MAPS)
		.iter()
		.find(|map| map.name == recent.map_name)
		.expect("Map should be in the cache.");

	let place = format!("[#{}]", GlobalAPI::get_place(recent.id, ctx.gokz_client()).await?);

	let (discord_timestamp, footer_msg) =
		match chrono::NaiveDateTime::parse_from_str(&recent.created_on, "%Y-%m-%dT%H:%M:%S") {
			Err(_) => (String::new(), String::new()),
			Ok(parsed_time) => (
				format!("<t:{}:R>", parsed_time.timestamp()),
				format!("{} GMT", parsed_time.format("%d/%m/%Y - %H:%M:%S")),
			),
		};

	let mode: Mode = recent.mode.parse()?;

	ctx.send(|reply| {
		reply.embed(|e| {
			e.color((116, 128, 194))
				.title(format!(
					"[PB] {} on {} (T{})",
					&recent.player_name.unwrap_or_else(|| String::from("unknown")),
					&map.name,
					&map.difficulty
				))
				.url(format!(
					"{}?{}=",
					formatting::map_link(&map.name),
					mode.short().to_lowercase()
				))
				.thumbnail(formatting::map_thumbnail(&map.name))
				.field(
					format!("{} {}", mode.short(), if recent.teleports > 0 { "TP" } else { "PRO" }),
					format!("> {} {}\n> {}{}", formatting::format_time(recent.time), place, discord_timestamp, {
						if recent.replay_id == 0 {
							String::new()
						} else {
							let link = GlobalAPI::get_replay_by_id(recent.replay_id);
							format!("\n> [Watch Replay](http://gokzmaptest.site.nfoservers.com/GlobalReplays/?replay={})\n> [Download Replay]({})",
								recent.replay_id,
								link
							)
						}
					}),
					true
				)
				.footer(|f| f.text(footer_msg).icon_url(crate::ICON))
		})
	})
	.await?;

	Ok(())
}
