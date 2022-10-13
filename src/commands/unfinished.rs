use std::env;

use bson::doc;
use gokz_rs::functions::{get_player, get_unfinished};
use gokz_rs::prelude::*;
use serenity::builder::CreateEmbed;
use serenity::model::user::User;
use serenity::{
	builder::CreateApplicationCommand,
	model::prelude::{
		command::CommandOptionType, interaction::application_command::CommandDataOption,
	},
};

use crate::util::{
	get_integer, get_string, is_mention, is_steamid, retrieve_mode, retrieve_steam_id, Target,
	UserSchema,
};
use crate::SchnoseCommand;

pub fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	cmd.name("unfinished")
		.description("Check which maps you still need to complete")
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
				.name("mode")
				.description("Specify a mode.")
				.add_string_choice("KZT", "kz_timer")
				.add_string_choice("SKZ", "kz_simple")
				.add_string_choice("VNL", "kz_vanilla")
				.required(false)
		})
		.create_option(|opt| {
			opt.kind(CommandOptionType::String)
				.name("runtype")
				.description("TP/PRO")
				.add_string_choice("TP", true)
				.add_string_choice("PRO", false)
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

	let tier = match get_integer("tier", opts) {
		Some(int) => Some(int as u8),
		None => None,
	};

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
			Err(why) => return SchnoseCommand::Message(why),
		}
	};

	let runtype = match get_string("runtype", opts) {
		Some(runtype) => {
			if runtype == "false" {
				false
			} else {
				true
			}
		}
		None => true,
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
				Err(why) => return SchnoseCommand::Message(why)
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
				Err(why) => return SchnoseCommand::Message(why),
			}
	};

	let player = match target {
		Target::SteamID(steam_id) => PlayerIdentifier::SteamId(steam_id),
		Target::Name(name) => match SteamId::get(&PlayerIdentifier::Name(name), &client).await {
			Ok(steam_id) => PlayerIdentifier::SteamId(steam_id),
			Err(why) => return SchnoseCommand::Message(why.tldr.to_owned()),
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
					Err(why) => return SchnoseCommand::Message(why),
				}
		}
	};

	let request = match get_unfinished(&player, &mode, runtype, tier, &client).await {
		Ok(map_names) => map_names,
		Err(why) => return SchnoseCommand::Message(why.tldr),
	};

	let player = match get_player(&player, &client).await {
		Ok(player) => player,
		Err(why) => return SchnoseCommand::Message(why.tldr),
	};

	let description = if request.len() <= 10 {
		request.join("\n")
	} else {
		format!(
			"{}\n...{} more",
			(request[0..10]).join("\n"),
			request.len() - 10
		)
	};

	let embed = CreateEmbed::default()
		.color((116, 128, 194))
		.title(format!(
			"Uncompleted Maps - {} {} {}",
			&mode.fancy_short(),
			if runtype { "TP" } else { "PRO" },
			if let Some(tier) = tier {
				format!("[T{}]", tier)
			} else {
				String::from(" ")
			}
		))
		.description(description)
		.footer(|f| {
			let icon_url = env::var("ICON").unwrap_or(String::from("unknown"));

			f.text(format!("Player: {}", player.name))
				.icon_url(icon_url)
		})
		.to_owned();

	SchnoseCommand::Embed(embed)
}
