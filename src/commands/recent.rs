use {
	crate::{
		events::slash_commands::{
			GlobalState,
			InteractionResponseData::{Message, Embed},
		},
		schnose::Target,
		util::format_time,
	},
	gokz_rs::{prelude::*, global_api::*},
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

pub(crate) async fn execute(mut state: GlobalState<'_>) -> anyhow::Result<()> {
	state.defer().await?;

	let target = Target::from(state.get::<String>("player"));
	let player = match target.to_player(state.user, state.db).await {
		Ok(player) => player,
		Err(why) => {
			log::warn!("[{}]: {} => {:?}", file!(), line!(), why);
			return state.reply(Message(&why)).await;
		},
	};

	let recent = match get_recent(&player, &state.req_client).await {
		Ok(record) => record,
		Err(why) => {
			log::warn!("[{}]: {} => {:?}", file!(), line!(), why);
			return state.reply(Message(&why.tldr)).await;
		},
	};

	let map = match get_map(&MapIdentifier::Name(recent.map_name.clone()), &state.req_client).await
	{
		Ok(map) => map,
		Err(why) => {
			log::warn!("[{}]: {} => {:?}", file!(), line!(), why);
			return state.reply(Message(&why.tldr)).await;
		},
	};

	let place = match get_place(&recent.id, &state.req_client).await {
		Ok(place) => format!("[#{}]", place.0),
		Err(why) => {
			log::warn!("[{}]: {} => {:?}", file!(), line!(), why);
			return state.reply(Message(&why.tldr)).await;
		},
	};

	let (discord_timestamp, fancy) =
		match chrono::NaiveDateTime::parse_from_str(&recent.created_on, "%Y-%m-%dT%H:%M:%S") {
			Ok(parsed_time) => (
				format!("<t:{}:R>", parsed_time.timestamp()),
				format!("{} GMT", parsed_time.format("%d/%m/%Y - %H:%M:%S")),
			),
			Err(_) => (String::from(" "), String::from(" ")),
		};

	let mode = Mode::from_str(&recent.mode).expect("This must be valid.");

	let embed = CreateEmbed::default()
		.color((116, 128, 194))
		.title(format!(
			"[PB] {} on {} (T{})",
			&recent.player_name.unwrap_or(String::from("unknown")),
			&map.name,
			&map.difficulty
		))
		.url(format!("https://kzgo.eu/maps/{}?{}=", &map.name, &mode.to_fancy()))
		.thumbnail(&state.thumbnail(&map.name))
		.field(
			format!("{} {}", mode.to_fancy(), if &recent.teleports > &0 { "TP" } else { "PRO" }),
			format!("> {} {}\n> {}{}", format_time(recent.time), place, discord_timestamp, {
				if &recent.replay_id != &0 {
					match get_replay(recent.replay_id).await {
						Ok(link) => format!("\n> [Download Replay]({})", link),
						Err(why) => {
							log::error!(
								"[{}]: {} => {}\n{:#?}",
								file!(),
								line!(),
								format!("Failed to get replay link for id {}", &recent.replay_id),
								why
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
		.footer(|f| f.text(fancy).icon_url(&state.icon))
		.to_owned();

	return state.reply(Embed(embed)).await;
}
