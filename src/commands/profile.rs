use gokz_rs::custom::get_profile;
use num_format::{ToFormattedString, Locale};

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
	gokz_rs::{prelude::*, global_api::*, kzgo},
	serenity::{
		builder::{CreateApplicationCommand, CreateEmbed},
		model::prelude::command::CommandOptionType,
	},
};

pub fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	return cmd
		.name("profile")
		.description("Check a player's stats.")
		.create_option(|opt| {
			opt.kind(CommandOptionType::String)
				.name("mode")
				.description("Specify a mode.")
				.add_string_choice("KZT", "kz_timer")
				.add_string_choice("SKZ", "kz_simple")
				.add_string_choice("VNL", "kz_vanilla")
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

	// sanitize user input
	let mode = match ctx.get_string("mode") {
		// mode has been specified
		Some(mode_name) => Mode::from_str(&mode_name).expect("`mode_name` has to be valid here."),
		// check db
		None => match ctx.db.find_one(doc! { "discordID": ctx.user.id.to_string() }, None).await {
			// user is in db
			Ok(doc) => match doc {
				Some(entry) => match entry.mode {
					Some(mode_name) if mode_name != "none" => Mode::from_str(&mode_name).expect("`mode_name` has to be valid here."),
					_ => return ctx.reply(Message("You either need to specify a mode or set a default option via `/mode`.")).await
				},
				// user not in db
				None => return ctx.reply(Message("You either need to specify a mode or set a default option via `/mode`.")).await
			},
			Err(why) => {
				log::error!(
					"[{}]: {} => {}\n{:?}",
					file!(), line!(), "Failed to access database.", why
				);
				return ctx.reply(Message("Failed to access database")).await;
			}
		}
	};

	let player = match sanitize_target(ctx.get_string("player"), &ctx.db, &ctx.user).await {
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

	let player_profile =
		match get_profile(&PlayerIdentifier::SteamID(steam_id), &mode, &client).await {
			Ok(profile) => profile,
			Err(why) => {
				log::error!(
					"[{}]: {} => {}\n{:?}",
					file!(),
					line!(),
					"Failed to fetch player profile.",
					why
				);
				return ctx.reply(Message("Failed to fetch player profile.")).await;
			},
		};

	let avatar = get_steam_avatar(&player_profile.steam_id64, &client).await;

	let fav_mode = match &player_profile.steam_id {
		Some(steam_id) => match ctx.db.find_one(doc! { "steamID": steam_id }, None).await {
			Ok(Some(entry)) => match entry.mode {
				Some(mode_name) if mode_name != "none" => Mode::from_str(&mode_name)
					.expect("`mode_name` has to be valid at this point.")
					.to_fancy(),
				_ => String::from("unknown"),
			},
			_ => String::from("unknown"),
		},
		None => String::from("unknown"),
	};

	let mut bars = [[""; 7]; 2].map(|a| a.map(|s| s.to_owned()));

	for i in 0..7 {
		{
			let amount = (&player_profile.completion_percentage[i].0 / 10.0) as u32;

			for _ in 0..amount {
				bars[0][i].push_str("█");
			}

			for _ in 0..(10 - amount) {
				bars[0][i].push_str("░");
			}
		}

		{
			let amount = (&player_profile.completion_percentage[i].1 / 10.0) as u32;

			for _ in 0..amount {
				bars[1][i].push_str("█");
			}

			for _ in 0..(10 - amount) {
				bars[1][i].push_str("░");
			}
		}
	}

	let doable = match kzgo::completion::get_completion_count(&mode, &client).await {
		Ok(data) => (data.tp.total, data.pro.total),
		Err(why) => {
			log::error!(
				"[{}]: {} => {}\n{:?}",
				file!(),
				line!(),
				"Failed to fetch completion count from KZGO.",
				why
			);
			return ctx.reply(Message(&why.tldr)).await;
		},
	};

	let embed = CreateEmbed::default()
		.color((116, 128, 194))
		.title(format!(
			"{} - {} Profile",
			&player_profile.name.unwrap_or(String::from("unknown")),
			&mode.to_fancy()
		))
		.url(format!(
			"https://kzgo.eu/players/{}?{}=",
			match &player_profile.steam_id {
				Some(steam_id) => steam_id,
				None => "unknown",
			},
			&mode.to_fancy().to_lowercase()
		))
		.thumbnail(avatar)
		.description(format!(
			r"
🏆 **World Records: {} (TP) | {} (PRO)**
────────────────────────
                      TP                                               PRO
         `{}/{} ({:.2}%)`              `{}/{} ({:.2}%)`
T1     ⌠ {} ⌡          ⌠ {} ⌡
T2   ⌠ {} ⌡          ⌠ {} ⌡
T3   ⌠ {} ⌡          ⌠ {} ⌡
T4  ⌠ {} ⌡          ⌠ {} ⌡
T5   ⌠ {} ⌡          ⌠ {} ⌡
T6  ⌠ {} ⌡          ⌠ {} ⌡
T7   ⌠ {} ⌡          ⌠ {} ⌡

Points: **{}**
────────────────────────
Rank: **{}**
Preferred Mode: {}
			",
			&player_profile.records.0,
			&player_profile.records.1,
			&player_profile.completion[7].0,
			&doable.0,
			&player_profile.completion_percentage[7].0,
			&player_profile.completion[7].1,
			&doable.1,
			&player_profile.completion_percentage[7].1,
			&bars[0][0],
			&bars[1][0],
			&bars[0][1],
			&bars[1][1],
			&bars[0][2],
			&bars[1][2],
			&bars[0][3],
			&bars[1][3],
			&bars[0][4],
			&bars[1][4],
			&bars[0][5],
			&bars[1][5],
			&bars[0][6],
			&bars[1][6],
			(&player_profile.points.0 + &player_profile.points.1).to_formatted_string(&Locale::en),
			match &player_profile.rank {
				Some(rank) => rank.to_string(),
				None => String::from("unknown"),
			},
			fav_mode
		))
		.footer(|f| {
			f.text(format!(
				"steamID: {}",
				&player_profile.steam_id.unwrap_or(String::from("unknown"))
			))
			.icon_url(&ctx.client.icon_url)
		})
		.to_owned();

	return ctx.reply(Embed(embed)).await;
}
