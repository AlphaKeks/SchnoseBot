use std::env;

use bson::doc;
use gokz_rs::functions::get_profile;
use gokz_rs::{kzgo, prelude::*};
use num_format::{Locale, ToFormattedString};
use serenity::builder::CreateEmbed;
use serenity::model::user::User;
use serenity::{
	builder::CreateApplicationCommand,
	model::prelude::{
		command::CommandOptionType, interaction::application_command::CommandDataOption,
	},
};

use crate::util::{
	get_steam_avatar, get_string, is_mention, is_steamid, retrieve_mode, retrieve_steam_id, Target,
	UserSchema,
};
use crate::SchnoseCommand;

pub fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	cmd.name("profile")
		.description("Check a player's completion and rank")
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
	user: &User,
	opts: &[CommandDataOption],
	mongo_client: &mongodb::Client,
) -> SchnoseCommand {
	let client = reqwest::Client::new();

	let mode = if let Some(mode_name) = get_string("mode", opts) {
		Mode::from(mode_name)
	} else {
		let collection = mongo_client
			.database("gokz")
			.collection::<UserSchema>("users");

		match retrieve_mode(doc! { "discordID": user.id.to_string() }, collection).await {
			Ok(mode) => match mode {
				Some(mode) => mode,
				None => {
					return SchnoseCommand::Message(String::from(
						"You need to specify a mode or set a default steamID with `/mode`.",
					))
				}
			},
			Err(why) => {
				tracing::error!("`retrieve_mode`: {:#?}", why);

				return SchnoseCommand::Message(why);
			}
		}
	};

	let target = if let Some(target) = get_string("target", opts) {
		if is_steamid(&target) {
			Target::SteamID(SteamId(target))
		} else if is_mention(&target) {
			let collection = mongo_client
				.database("gokz")
				.collection::<UserSchema>("users");

			let id;
			if let Some(s) = target.split_once(">") {
				if let Some(s) = s.0.split_once("@") {
					id = s.1.to_owned();
				} else {
					id = String::new();
				}
			} else {
				id = String::new();
			}

			match retrieve_steam_id(id, collection).await {
				Ok(steam_id) => match steam_id {
					Some(steam_id) => Target::SteamID(steam_id),
					None => return SchnoseCommand::Message(String::from("You need to provide a target (steamID, name or mention) or set a default steamID with `/setsteam`."))
				}
				Err(why) => {
					tracing::error!("`retrieve_steam_id`: {:#?}", why);

					return SchnoseCommand::Message(why)
				}
			}
		} else {
			Target::Name(target)
		}
	} else {
		let collection = mongo_client
			.database("gokz")
			.collection::<UserSchema>("users");

		match retrieve_steam_id(user.id.to_string(), collection).await {
				Ok(steam_id) => match steam_id {
					Some(steam_id) => Target::SteamID(steam_id),
					None => return SchnoseCommand::Message(String::from("You need to provide a target (steamID, name or mention) or set a default steamID with `/setsteam`."))
				},
				Err(why) => {
					tracing::error!("`retrieve_steam_id`: {:#?}", why);

					return SchnoseCommand::Message(why)
				}
			}
	};

	let player = match target {
		Target::SteamID(steam_id) => PlayerIdentifier::SteamId(steam_id),
		Target::Name(name) => match SteamId::get(&PlayerIdentifier::Name(name), &client).await {
			Ok(steam_id) => PlayerIdentifier::SteamId(steam_id),
			Err(why) => {
				tracing::error!("`SteamId::get()`: {:#?}", why);

				return SchnoseCommand::Message(why.tldr.to_owned());
			}
		},
		Target::Mention(mention) => {
			let collection = mongo_client
				.database("gokz")
				.collection::<UserSchema>("users");

			match retrieve_steam_id(mention, collection).await {
					Ok(steam_id) => match steam_id {
						Some(steam_id) => PlayerIdentifier::SteamId(steam_id),
						None => return SchnoseCommand::Message(String::from(
							"You need to provide a target (steamID, name or mention) or set a default steamID with `/setsteam`.",
						)),
					},
					Err(why) => {
						tracing::error!("`retrieve_steam_id`: {:#?}", why);

						return SchnoseCommand::Message(why)
					},
				}
		}
	};

	let profile = match get_profile(&player, &mode, &client).await {
		Ok(profile) => profile,
		Err(why) => {
			tracing::error!("`get_profile`: {:#?}", why);

			return SchnoseCommand::Message(why.tldr);
		}
	};

	let picture = get_steam_avatar(&profile.steam_id64, &client).await;

	let pref_mode = if let Some(steam_id) = &profile.steam_id {
		let collection = mongo_client
			.database("gokz")
			.collection::<UserSchema>("users");

		match retrieve_mode(doc! { "steamID": steam_id }, collection).await {
			Ok(mode) => match mode {
				Some(mode) => mode.fancy(),
				None => String::from("unknown"),
			},
			Err(_) => String::from("unknown"),
		}
	} else {
		String::from("unknown")
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

	let doable = match kzgo::completion::get_completion_count(&mode, &client).await {
		Ok(data) => (data.tp.total, data.pro.total),
		Err(why) => {
			tracing::error!("`kzgo::completion::get_completion_count()`: {:#?}", why);

			return SchnoseCommand::Message(why.tldr);
		}
	};

	let embed = CreateEmbed::default()
		.color((116, 128, 194))
		.title(format!(
			"{} - {} Profile",
			match &profile.name {
				Some(name) => name,
				None => "unknown",
			},
			&mode.fancy()
		))
		.url(format!(
			"https://kzgo.eu/players/{}?{}=",
			match &profile.steam_id {
				Some(steam_id) => steam_id,
				None => "",
			},
			&mode.fancy_short().to_lowercase()
		))
		.thumbnail(picture)
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
			pref_mode
		))
		// .description("hi")
		.footer(|f| {
			let icon_url = env::var("ICON").unwrap_or(String::from("unknown"));

			f.text(format!(
				"steamID: {}",
				match &profile.steam_id {
					Some(steam_id) => steam_id,
					None => "unknown",
				}
			))
			.icon_url(icon_url)
		})
		.to_owned();

	SchnoseCommand::Embed(embed)
}
