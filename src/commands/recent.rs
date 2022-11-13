use {
	crate::{
		events::slash_command::{
			InteractionData,
			InteractionResponseData::{Message, Embed},
		},
		util::*,
	},
	anyhow::Result,
	gokz_rs::{prelude::*, global_api::*},
	serenity::{
		builder::{CreateApplicationCommand, CreateEmbed},
		model::prelude::command::CommandOptionType,
	},
};

pub fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
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

pub async fn execute(mut ctx: InteractionData<'_>) -> Result<()> {
	ctx.defer().await?;

	// sanitize user input
	let player = match sanitize_target(ctx.get_string("player"), &ctx.db, &ctx.root).await {
		Some(target) => target,
		None => {
			return ctx
				.reply(Message("Please specify a player or save your own SteamID via `/setsteam`."))
				.await
		},
	};

	let client = reqwest::Client::new();

	let steam_id = match player {
		Target::SteamID(steam_id) => steam_id,
		Target::Name(player_name) => {
			match get_player(&PlayerIdentifier::Name(player_name), &client).await {
				Ok(player) => SteamID(player.steam_id),
				Err(why) => {
					log::warn!(
						"[{}]: {} => {}\n{:?}",
						file!(),
						line!(),
						"Failed to get player from the GlobalAPI.",
						why
					);
					return ctx.reply(Message("Couldn't fetch player from GlobalAPI.")).await;
				},
			}
		},
	};

	let recent = match get_recent(&PlayerIdentifier::SteamID(steam_id), &client).await {
		Ok(record) => record,
		Err(why) => {
			log::error!("[{}]: {} => {}\n{:?}", file!(), line!(), "Failed to fetch recent.", why);
			return ctx.reply(Message(&why.tldr)).await;
		},
	};

	let map = match get_map(&MapIdentifier::Name(recent.map_name.clone()), &client).await {
		Ok(map) => map,
		Err(why) => {
			log::error!("[{}]: {} => {}\n{:?}", file!(), line!(), "Failed to get place.", why);
			return ctx.reply(Message(&why.tldr)).await;
		},
	};

	let mode = Mode::from_str(&recent.mode)
		.expect("If this is invalid, go complain to the GlobalAPI team.");

	let place = match get_place(&recent.id, &client).await {
		Ok(place) => format!("[#{}]", place.0),
		Err(why) => {
			log::error!("[{}]: {} => {}\n{:?}", file!(), line!(), "Failed to get place.", why);
			return ctx.reply(Message(&why.tldr)).await;
		},
	};

	let (discord_timestamp, fancy) =
		match chrono::NaiveDateTime::parse_from_str(&recent.created_on, "%Y-%m-%dT%H:%M:%S") {
			Ok(parsed_time) => (
				format!("<t:{}:R>", parsed_time.timestamp()),
				format!("{} GMT", parsed_time.format("%d/%m/%Y - %H:%H:%S")),
			),
			Err(_) => (String::from(" "), String::from(" ")),
		};

	let embed = CreateEmbed::default()
		.color((116, 128, 194))
		.title(format!(
			"[PB] {} on {} (T{})",
			&recent.player_name.unwrap_or(String::from("unknown")),
			&map.name,
			&map.difficulty
		))
		.url(format!("https://kzgo.eu/maps/{}?{}=", &map.name, &mode.to_fancy()))
		.thumbnail(format!(
			"https://raw.githubusercontent.com/KZGlobalTeam/map-images/master/images/{}.jpg",
			&map.name
		))
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
		.footer(|f| f.text(fancy).icon_url(&ctx.client.icon_url))
		.to_owned();

	return ctx.reply(Embed(embed)).await;
}
