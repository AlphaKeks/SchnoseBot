use {
	crate::{
		prelude::{InteractionResult, Target},
		events::interactions::InteractionState,
		formatting::format_time,
	},
	gokz_rs::{prelude::*, global_api::*},
	log::error,
	serenity::{
		builder::{CreateApplicationCommand, CreateEmbed},
		model::prelude::command::CommandOptionType,
	},
};

pub(crate) fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	return cmd
		.name("recent")
		.description("Check a player's most recent personal best.")
		.create_option(|opt| {
			opt.kind(CommandOptionType::String)
				.name("player")
				.description("Specify a player.")
				.required(false)
		});
}

pub(crate) async fn execute(state: &mut InteractionState<'_>) -> InteractionResult {
	// Defer current interaction since this could take a while
	state.defer().await?;

	let target = Target::from(state.get::<String>("player"));

	let player = target.into_player(state.user, state.db).await?;

	let recent = get_recent(&player, state.req_client).await?;

	let map = get_map(&MapIdentifier::Name(recent.map_name.clone()), state.req_client).await?;

	let place = get_place(&recent.id, state.req_client).await?;
	let place = format!("[#{}]", place.0);

	let (discord_timestamp, footer_msg) =
		match chrono::NaiveDateTime::parse_from_str(&recent.created_on, "%Y-%m-%dT%H:%M:%S") {
			Ok(parsed_time) => (
				format!("<t:{}:R>", parsed_time.timestamp()),
				format!("{} GMT", parsed_time.format("%d/%m/%Y - %H:%M:%S")),
			),
			Err(_) => (String::from(" "), String::from(" ")),
		};

	let mode = &recent.mode.parse::<Mode>().expect("This must be valid.");

	let embed = CreateEmbed::default()
		.color((116, 128, 194))
		.title(format!(
			"[PB] {} on {} (T{})",
			&recent.player_name.unwrap_or_else(|| String::from("unknown")),
			&map.name,
			&map.difficulty
		))
		.url(format!("{}?{}=", state.map_link(&map.name), &mode.to_fancy().to_lowercase()))
		.thumbnail(&state.map_thumbnail(&map.name))
		.field(
			format!("{} {}", mode.to_fancy(), if recent.teleports > 0 { "TP" } else { "PRO" }),
			format!("> {} {}\n> {}{}", format_time(recent.time), place, discord_timestamp, {
				if recent.replay_id != 0 {
					match get_replay(recent.replay_id).await {
						Ok(link) => format!(
							"\n> [Watch Replay]({})\n> [Download Replay]({})",
							format_args!(
								"http://gokzmaptest.site.nfoservers.com/GlobalReplays/?replay={}",
								&recent.replay_id
							),
							link
						),
						Err(why) => {
							error!(
								"Failed to get replay link for id {}: {:?}",
								&recent.replay_id, why
							);
							String::new()
						},
					}
				} else {
					String::new()
				}
			}),
			true,
		)
		.footer(|f| f.text(footer_msg).icon_url(state.icon))
		.to_owned();

	Ok(embed.into())
}
