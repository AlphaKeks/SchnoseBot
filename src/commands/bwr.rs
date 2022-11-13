use {
	crate::{
		events::slash_command::{
			InteractionData,
			InteractionResponseData::{Message, Embed},
		},
		util::*,
	},
	anyhow::Result,
	bson::doc,
	itertools::Itertools,
	gokz_rs::{prelude::*, global_api::*},
	futures::future::join_all,
	serenity::{
		builder::{CreateApplicationCommand, CreateEmbed},
		model::prelude::command::CommandOptionType,
	},
};

pub fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	return cmd
		.name("bwr")
		.description("Check a World Record on a bonus.")
		.create_option(|opt| {
			opt.kind(CommandOptionType::String)
				.name("map_name")
				.description("Specify a map.")
				.required(true)
		})
		.create_option(|opt| {
			opt.kind(CommandOptionType::String)
				.name("mode")
				.description("Choose a mode.")
				.add_string_choice("KZT", "kz_timer")
				.add_string_choice("SKZ", "kz_simple")
				.add_string_choice("VNL", "kz_vanilla")
				.required(false)
		})
		.create_option(|opt| {
			opt.kind(CommandOptionType::Integer)
				.name("course")
				.description("Specify a course.")
				.required(false)
		});
}

pub async fn execute(mut ctx: InteractionData<'_>) -> Result<()> {
	ctx.defer().await?;

	// sanitize user input
	let map = ctx.get_string("map_name").expect("This option is marked as `required`.");
	let mode = match ctx.get_string("mode") {
		Some(mode_name) => Mode::from_str(&mode_name)
			.expect("`mode_name` _has_ to be valid here. See the `register` function above."),
		None => match ctx.db.find_one(doc! { "discordID": ctx.user.id.to_string() }, None).await {
			Ok(Some(entry)) => Mode::from_str(&entry.mode.unwrap_or(String::from("kz_timer")))
				.expect("Mode stored in the database _needs_ to be valid."),
			_ => Mode::KZTimer,
		},
	};
	let course = match ctx.get_int("course") {
		Some(course) => course as u8,
		None => 1,
	};

	let client = reqwest::Client::new();

	let global_maps = match get_maps(&client).await {
		Ok(maps) => maps,
		Err(why) => {
			log::error!(
				"[{}]: {} => {}\n{:?}",
				file!(),
				line!(),
				"Failed to fetch global maps.",
				why
			);
			return ctx.reply(Message("Failed to fetch global maps.")).await;
		},
	};

	let map = match is_global(&MapIdentifier::Name(map), &global_maps).await {
		Ok(map) => map,
		Err(why) => {
			log::warn!("[{}]: {} => {}\n{:?}", file!(), line!(), "Given map is not global.", why);
			return ctx.reply(Message("Please input a global map.")).await;
		},
	};

	let map_identifier = MapIdentifier::Name(map.name.clone());

	let (tp, pro) = join_all([
		get_wr(&map_identifier, &mode, true, course, &client),
		get_wr(&map_identifier, &mode, false, course, &client),
	])
	.await
	.into_iter()
	.collect_tuple()
	.unwrap();

	if let (&Err(_), &Err(_)) = (&tp, &pro) {
		return ctx.reply(Message("No WRs found 😔")).await;
	}

	let mut embed = CreateEmbed::default()
		.color((116, 128, 194))
		.title(format!("[BWR {}] {} (T{})", &course, &map.name, &map.difficulty))
		.url(format!(
			"https://kzgo.eu/maps/{}?bonus={}&{}=",
			&map.name,
			&course,
			&mode.to_fancy().to_lowercase()
		))
		.thumbnail(format!(
			"https://raw.githubusercontent.com/KZGlobalTeam/map-images/master/images/{}.jpg",
			&map.name
		))
		.field(
			"TP",
			format!(
				"{} {}",
				match &tp {
					Ok(rec) => format_time(rec.time),
					Err(_) => String::from("😔"),
				},
				match &tp {
					Ok(rec) => format!(
						"({})",
						match &rec.player_name {
							Some(name) => name,
							None => "unknown",
						}
					),
					Err(_) => String::from("unknown"),
				}
			),
			true,
		)
		.field(
			"PRO",
			format!(
				"{} {}",
				match &pro {
					Ok(rec) => format_time(rec.time),
					Err(_) => String::from("😔"),
				},
				match &pro {
					Ok(rec) => format!(
						"({})",
						match &rec.player_name {
							Some(name) => name,
							None => "unknown",
						}
					),
					Err(_) => String::from("unknown"),
				}
			),
			true,
		)
		.footer(|f| f.text(format!("Mode: {}", mode.to_fancy())).icon_url(&ctx.client.icon_url))
		.to_owned();

	let (tp_link, pro_link) = {
		let (mut tp_link, mut pro_link) = (String::new(), String::new());

		if let Ok(rec) = &tp {
			if rec.replay_id != 0 {
				if let Ok(link) = get_replay(rec.replay_id).await {
					tp_link = link;
				}
			}
		}

		if let Ok(rec) = &pro {
			if rec.replay_id != 0 {
				if let Ok(link) = get_replay(rec.replay_id).await {
					pro_link = link;
				}
			}
		}

		(tp_link, pro_link)
	};

	let tp = tp_link.len() > 0;
	let pro = pro_link.len() > 0;
	if tp || pro {
		let mut description = String::from("Download Replays:");

		if tp && !pro {
			description.push_str(&format!(" [TP]({})", tp_link))
		} else if !tp && pro {
			description.push_str(&format!(" [PRO]({})", pro_link))
		} else {
			description.push_str(&format!(" [TP]({}) | [PRO]({})", tp_link, pro_link))
		}

		embed.description(description);
	}

	return ctx.reply(Embed(embed)).await;
}
