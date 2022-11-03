use gokz_rs::{
	global_api::{get_player, get_unfinished},
	prelude::*,
};
use serenity::{
	builder::{CreateApplicationCommand, CreateEmbed},
	model::prelude::command::CommandOptionType,
};

use bson::doc;

use crate::{
	event_handler::interaction_create::{Metadata, SchnoseResponseData},
	util::{
		get_id_from_mention, retrieve_mode, retrieve_steam_id, sanitize_target, Target, UserSchema,
	},
};

pub fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	cmd.name("unfinished")
		.description("Check which maps a player has to complete.")
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
			Ok(mode) => Some(mode),
		},
		None => {
			match retrieve_mode(doc! { "discordID": metadata.cmd.user.id.to_string() }, collection)
				.await
			{
				Err(why) => {
					log::error!("[{}]: {} => {}", file!(), line!(), why,);

					return metadata
						.reply(SchnoseResponseData::Message(String::from(
							"You must either specify a mode or set a default one with `/mode`.",
						)))
						.await;
				},
				Ok(mode) => mode,
			}
		},
	};
	let runtype_input = match metadata.opts.get_string("runtype") {
		None => true,
		Some(runtype) => match runtype.as_str() {
			"true" => true,
			"false" => false,
			_ => unreachable!("only `true` and `false` exist"),
		},
	};
	let tier_input = match metadata.opts.get_int("tier") {
		None => None,
		Some(tier) => Some(tier as u8),
	};
	let target_input = match metadata.opts.get_string("target") {
		Some(target) => sanitize_target(target),
		None => Target::None,
	};

	let client = reqwest::Client::new();

	let mode = match mode_input {
		Some(mode) => mode,
		None => {
			log::error!("[{}]: {} => {}", file!(), line!(), "No mode specified.",);

			return metadata
				.reply(SchnoseResponseData::Message(String::from(
					"You must either specify a mode or set a default one with `/mode`.",
				)))
				.await;
		},
	};

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

	let description = match get_unfinished(
		&PlayerIdentifier::SteamID(steam_id),
		&mode,
		runtype_input,
		tier_input,
		&client,
	)
	.await
	{
		Err(why) => {
			log::error!(
				"[{}]: {} => {}\n{:#?}",
				file!(),
				line!(),
				"Failed to get unfinished maps.",
				why
			);
			return metadata.reply(SchnoseResponseData::Message(why.tldr)).await;
		},
		Ok(list) => {
			if list.len() <= 10 {
				list.join("\n")
			} else {
				format!("{}\n...{} more", (list[0..10]).join("\n"), list.len() - 10)
			}
		},
	};

	let embed = CreateEmbed::default()
		.color((116, 128, 194))
		.title(format!(
			"Uncompleted Maps - {} {} {}",
			&mode.to_fancy(),
			if runtype_input { "TP" } else { "PRO" },
			match tier_input {
				Some(tier) => format!("[T{}]", tier),
				None => String::new(),
			}
		))
		.description(description)
		.footer(|f| f.text(format!("Mode: {}", mode.to_fancy())).icon_url(&root.icon))
		.to_owned();

	return metadata.reply(SchnoseResponseData::Embed(embed)).await;
}
