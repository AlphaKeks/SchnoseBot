use crate::{
	event_handler::interaction_create::{Metadata, SchnoseResponseData},
	util::{
		get_id_from_mention, get_steam_avatar, retrieve_mode, retrieve_steam_id, sanitize_target,
		Target, UserSchema,
	},
};

use bson::doc;

use gokz_rs::{custom::get_profile, global_api::get_player, kzgo, prelude::*};

use num_format::{Locale, ToFormattedString};
use serenity::{
	builder::{CreateApplicationCommand, CreateEmbed},
	model::prelude::command::CommandOptionType,
};

pub fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	cmd.name("profile")
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
				.name("target")
				.description("Specify a target.")
				.required(false)
		})
}

pub async fn run(
	metadata: Metadata,
	collection: &mongodb::Collection<UserSchema>,
	root: &crate::Schnose,
) {
	// sanitize user input
	let mode_input = match metadata.opts.get_string("mode") {
		Some(mode_name) => match Mode::from_str(&mode_name) {
			Err(why) => {
				log::error!(
					"[{}]: {} => {}\n{:#?}",
					file!(),
					line!(),
					"Failed to parse mode.",
					why
				);
				return metadata.reply(SchnoseResponseData::Message(why.tldr)).await;
			},
			Ok(mode) => mode,
		},
		None => {
			match retrieve_mode(doc! { "discordID": metadata.cmd.user.id.to_string() }, collection)
				.await
			{
				Err(why) => {
					log::error!("[{}]: {} => {}", file!(), line!(), why);
					return metadata
						.reply(SchnoseResponseData::Message(String::from(
							"You must either specify a mode or set a default one with `/mode`.",
						)))
						.await;
				},
				Ok(mode) => {
					match mode {
						None => {
							log::error!(
								"[{}]: {} => {}",
								file!(),
								line!(),
								"No mode found in database."
							);
							return metadata.reply(SchnoseResponseData::Message(String::from(
								"You must either specify a mode or set a default one with `/mode`.",
							))).await;
						},
						Some(mode) => mode,
					}
				},
			}
		},
	};
	let target_input = match metadata.opts.get_string("target") {
		Some(target) => sanitize_target(target),
		None => Target::None,
	};

	let client = reqwest::Client::new();

	let steam_id = match target_input {
		Target::None => {
			match retrieve_steam_id(
				doc! { "discordID": metadata.cmd.user.id.to_string() },
				collection,
			)
			.await
			{
				Err(why) => {
					log::error!("[{}]: {} => {}", file!(), line!(), why,);
					return metadata.reply(SchnoseResponseData::Message(String::from(
						"You must either specify a target or save your SteamID with `/setsteam`.",
					))).await;
				},
				Ok(steam_id) => match steam_id {
					Some(steam_id) => steam_id,
					None => {
						log::error!("[{}]: {} => {}", file!(), line!(), "Failed to parse mode.",);
						return metadata.reply(SchnoseResponseData::Message(String::from(
							"You must either specify a target or save your SteamID with `/setsteam`.",
						))).await;
					},
				},
			}
		},
		Target::Mention(mention) => match get_id_from_mention(mention) {
			Ok(id) => match retrieve_steam_id(doc! { "discordID": id.to_string() }, collection)
				.await
			{
				Err(why) => {
					log::error!("[{}]: {} => {}", file!(), line!(), why,);
					return metadata
						.reply(SchnoseResponseData::Message(String::from(
							"The person you @metion'd didn't save their SteamID in the database.",
						)))
						.await;
				},
				Ok(steam_id) => match steam_id {
					Some(steam_id) => steam_id,
					None => {
						log::error!("[{}]: {} => {}", file!(), line!(), "No SteamID specified.",);
						return metadata.reply(SchnoseResponseData::Message(String::from(
							"The person you @metion'd didn't save their SteamID in the database.",
						))).await;
					},
				},
			},
			Err(why) => {
				log::error!("[{}]: {} => {}", file!(), line!(), why);
				return metadata.reply(SchnoseResponseData::Message(why)).await;
			},
		},
		Target::SteamID(steam_id) => steam_id,
		Target::Name(input_name) => {
			match get_player(&PlayerIdentifier::Name(input_name), &client).await {
				Err(why) => {
					log::error!(
						"[{}]: {} => {}\n{:#?}",
						file!(),
						line!(),
						"Failed to get player from GlobalAPI",
						why
					);
					return metadata.reply(SchnoseResponseData::Message(why.tldr)).await;
				},
				Ok(player) => SteamID(player.steam_id),
			}
		},
	};

	let profile =
		match get_profile(&PlayerIdentifier::SteamID(steam_id), &mode_input, &client).await {
			Err(why) => {
				log::error!(
					"[{}]: {} => {}\n{:#?}",
					file!(),
					line!(),
					"Failed to get player profile.",
					why
				);
				return metadata.reply(SchnoseResponseData::Message(why.tldr)).await;
			},
			Ok(profile) => profile,
		};

	let avatar = get_steam_avatar(&profile.steam_id64, &client).await;

	let mode_pref = match &profile.steam_id {
		Some(steam_id) => match retrieve_mode(doc! { "steamID": steam_id }, collection).await {
			Ok(mode) => match mode {
				Some(mode) => mode.to_fancy(),
				None => String::from("unknown"),
			},
			Err(_) => String::from("unknown"),
		},
		None => String::from("unknown"),
	};

	let mut bars = [[""; 7]; 2].map(|a| a.map(|s| s.to_owned()));

	for i in 0..7 {
		{
			let amount = (&profile.completion_percentage[i].0 / 10.0) as u32;

			for _ in 0..amount {
				bars[0][i].push_str("█");
			}

			for _ in 0..(10 - amount) {
				bars[0][i].push_str("░");
			}
		}

		{
			let amount = (&profile.completion_percentage[i].1 / 10.0) as u32;

			for _ in 0..amount {
				bars[1][i].push_str("█");
			}

			for _ in 0..(10 - amount) {
				bars[1][i].push_str("░");
			}
		}
	}

	let doable = match kzgo::completion::get_completion_count(&mode_input, &client).await {
		Ok(data) => (data.tp.total, data.pro.total),
		Err(why) => {
			log::error!("`kzgo::completion::get_completion_count()`: {:#?}", why);
			return metadata.reply(SchnoseResponseData::Message(why.tldr)).await;
		},
	};

	let embed = CreateEmbed::default()
		.color((116, 128, 194))
		.title(format!(
			"{} - {} Profile",
			match &profile.name {
				Some(name) => name,
				None => "unknown",
			},
			&mode_input.to_fancy()
		))
		.url(format!(
			"https://kzgo.eu/players/{}?{}=",
			match &profile.steam_id {
				Some(steam_id) => steam_id,
				None => "",
			},
			&mode_input.to_fancy().to_lowercase()
		))
		.thumbnail(avatar)
		.description(format!(
			r"
🏆 **World Records: {} (TP) || {} (PRO)**
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
			&profile.records.0,
			&profile.records.1,
			&profile.completion[7].0,
			&doable.0,
			&profile.completion_percentage[7].0,
			&profile.completion[7].1,
			&doable.1,
			&profile.completion_percentage[7].1,
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
			(&profile.points.0 + &profile.points.1).to_formatted_string(&Locale::en),
			match &profile.rank {
				Some(rank) => rank.to_string(),
				None => String::from("unknown"),
			},
			mode_pref
		))
		.footer(|f| {
			f.text(format!(
				"steamID: {}",
				match &profile.steam_id {
					Some(steam_id) => steam_id,
					None => "unknown",
				}
			))
			.icon_url(&root.icon)
		})
		.to_owned();

	return metadata.reply(SchnoseResponseData::Embed(embed)).await;
}
