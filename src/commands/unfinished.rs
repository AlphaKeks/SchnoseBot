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
	gokz_rs::{prelude::*, global_api::*},
	serenity::{
		builder::{CreateApplicationCommand, CreateEmbed},
		model::prelude::command::CommandOptionType,
	},
};

pub fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	return cmd
		.name("unfinished")
		.description("Check which maps a player still needs to complete.")
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
			opt.kind(CommandOptionType::String)
				.name("runtype")
				.description("TP/PRO")
				.add_string_choice("TP", "true")
				.add_string_choice("PRO", "false")
				.required(false)
		})
		.create_option(|opt| {
			opt.kind(CommandOptionType::Integer)
				.name("tier")
				.description("Filter by tier")
				.add_int_choice("1 (Very Easy)", 1)
				.add_int_choice("2 (Easy)", 2)
				.add_int_choice("3 (Medium)", 3)
				.add_int_choice("4 (Hard)", 4)
				.add_int_choice("5 (Very Hard)", 5)
				.add_int_choice("6 (Extreme)", 6)
				.add_int_choice("7 (Death)", 7)
				.required(false)
		})
		.create_option(|opt| {
			opt.kind(CommandOptionType::String)
				.name("player")
				.description("Specify a player.")
				.required(false)
		});
}

pub async fn execute(mut ctx: InteractionData<'_>) -> Result<()> {
	ctx.defer().await?;

	let mode =
		match ctx.get_string("mode") {
			Some(mode_name) => {
				Mode::from_str(&mode_name).expect("`mode_name` has to be valid at this point.")
			},
			None => {
				match ctx.db.find_one(doc! { "discordID": ctx.user.id.to_string() }, None).await {
				Ok(document) => match document {
					Some(entry) => match entry.mode {
						Some(mode_name) if mode_name != "none" => Mode::from_str(&mode_name)
							.expect("`mode_name` has to be valid at this point."),
						_ => return ctx.reply(Message(
							"You either need to specify a mode or set a default one via `/mode`.",
						)).await,
					},
					None => {
						return ctx.reply(Message(
							"You either need to specify a mode or set a default one via `/mode`.",
						)).await
					},
				},
				Err(why) => {
					log::error!(
						"[{}]: {} => {}\n{:?}",
						file!(),
						line!(),
						"Failed to access database.",
						why
					);
					return ctx.reply(Message("Failed to access database.")).await;
				},
			}
			},
		};
	let runtype = match ctx.get_string("runtype") {
		Some(runtype) => match runtype.as_str() {
			"true" => true,
			"false" => false,
			_ => unreachable!("only `true` and `false` exist as options"),
		},
		None => true,
	};
	let tier = match ctx.get_int("tier") {
		Some(tier) => Some(tier as u8),
		None => None,
	};

	let client = reqwest::Client::new();

	let steam_id = match sanitize_target(ctx.get_string("player"), &ctx.db, &ctx.user).await {
		Some(target) => match target {
			Target::Name(name) => match get_player(&PlayerIdentifier::Name(name), &client).await {
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
			},
			Target::SteamID(steam_id) => steam_id
		},
		// check db
		None => match ctx.db.find_one(doc! { "discordID": ctx.user.id.to_string() }, None).await {
			Ok(doc) => match doc {
				Some(entry) => match entry.steamID {
					Some(steam_id) => SteamID(steam_id),
					None => return ctx.reply(Message("You either need to specify a player or save your own SteamID via `/setsteam`.")).await
				},
				None => return ctx.reply(Message("You either need to specify a player or save your own SteamID via `/setsteam`.")).await
			},
			Err(why) => {
				log::error!(
					"[{}]: {} => {}\n{:?}",
					file!(), line!(), "Failed to access database.", why
				);
				return ctx.reply(Message("Failed to access database.")).await;
			}
		},
	};

	let player_identifier = PlayerIdentifier::SteamID(steam_id);

	let player_name = match get_player(&player_identifier, &client).await {
		Ok(player) => player.name,
		Err(why) => {
			log::error!("[{}]: {} => {}\n{:?}", file!(), line!(), "Failed to fetch player.", why);
			String::from("unknown")
		},
	};

	let (description, amount) =
		match get_unfinished(&player_identifier, &mode, runtype, tier, &client).await {
			Ok(map_list) => {
				let description = if map_list.len() <= 10 {
					map_list.join("\n")
				} else {
					format!("{}\n...{} more", (map_list[0..10]).join("\n"), map_list.len() - 10)
				};
				let amount = if map_list.len() == 1 {
					format!("{} uncompleted map", map_list.len())
				} else {
					format!("{} uncompleted maps", map_list.len())
				};
				(description, amount)
			},
			Err(why) => {
				log::error!(
					"[{}]: {} => {}\n{:?}",
					file!(),
					line!(),
					"Failed to fetch unfinished maps.",
					why
				);
				return ctx.reply(Message(&why.tldr)).await;
			},
		};

	let embed = CreateEmbed::default()
		.color((116, 128, 194))
		.title(format!(
			"{} - {} {} {}",
			amount,
			&mode.to_fancy(),
			if runtype { "TP" } else { "PRO" },
			match tier {
				Some(tier) => format!("[T{}]", tier),
				None => String::new(),
			}
		))
		.description(description)
		.footer(|f| f.text(format!("Player: {}", player_name)).icon_url(&ctx.client.icon_url))
		.to_owned();

	return ctx.reply(Embed(embed)).await;
}
