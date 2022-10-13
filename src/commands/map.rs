use std::env;

use gokz_rs::{
	functions::{get_filters, get_maps, is_global},
	kzgo,
	prelude::MapIdentifier,
};
use serenity::{
	builder::{CreateApplicationCommand, CreateEmbed},
	model::prelude::{
		command::CommandOptionType, interaction::application_command::CommandDataOption,
	},
};

use crate::{util::get_string, SchnoseCommand};

pub fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	cmd.name("map")
		.description("Get detailed information on a map")
		.create_option(|opt| {
			opt.kind(CommandOptionType::String)
				.name("map_name")
				.description("Specify a map.")
				.required(true)
		})
}

pub async fn run(opts: &[CommandDataOption]) -> SchnoseCommand {
	let client = reqwest::Client::new();

	let (global_api, kzgo) = match get_string("map_name", opts) {
		Some(map_name) => {
			let global_maps = match get_maps(&client).await {
				Ok(maps) => maps,
				Err(why) => {
					tracing::error!("`get_maps`: {:#?}", why);

					return SchnoseCommand::Message(why.tldr);
				}
			};

			let global_api = match is_global(&MapIdentifier::Name(map_name), &global_maps).await {
				Ok(map) => map,
				Err(why) => {
					tracing::error!("`is_global`: {:#?}", why);

					return SchnoseCommand::Message(why.tldr);
				}
			};

			let kzgo =
				match kzgo::maps::get_map(&MapIdentifier::Name(global_api.name.clone()), &client)
					.await
				{
					Ok(map) => map,
					Err(why) => {
						tracing::error!("`kzgo::maps::get_map`: {:#?}", why);

						return SchnoseCommand::Message(why.tldr);
					}
				};

			(global_api, kzgo)
		}
		None => unreachable!("Failed to access required command option"),
	};

	let mappers = {
		let mut v = vec![];
		for i in 0..kzgo.mapperNames.len() {
			v.push(format!(
				"[{}](https://steamcommunity.com/profiles/{})",
				kzgo.mapperNames[i], kzgo.mapperIds[i]
			))
		}

		v.join(", ")
	};

	let filters = match get_filters(&MapIdentifier::Id(global_api.id), &client).await {
		Ok(filters) => {
			let filters = filters
				.into_iter()
				.filter(|f| f.stage == 0)
				.collect::<Vec<gokz_rs::global_api::record_filters::base::Response>>();

			let mut res = ("❌", "❌", "❌");

			for filter in filters {
				match filter.mode_id {
					200 => res.0 = "✅",
					201 => res.1 = "✅",
					202 => res.2 = "✅",
					_ => (),
				}
			}

			res
		}
		Err(why) => {
			tracing::error!("`get_filters`: {:#?}", why);

			return SchnoseCommand::Message(why.tldr);
		}
	};

	let embed = CreateEmbed::default()
		.color((116, 128, 194))
		.title(&global_api.name)
		.url(format!("https://kzgo.eu/maps/{}", &global_api.name,))
		.thumbnail(format!(
			"https://raw.githubusercontent.com/KZGlobalTeam/map-images/master/images/{}.jpg",
			&global_api.name
		))
		.description(format!(
			r"
			🢂 API Tier: {}
			🢂 Mapper(s): {}
			🢂 Bonuses: {}
			🢂 Global Date: {}

			🢂 Filters:
			",
			&global_api.difficulty,
			mappers,
			&kzgo.bonuses,
			match chrono::NaiveDateTime::parse_from_str(&global_api.updated_on, "%Y-%m-%dT%H:%M:%S")
			{
				Ok(parsed_time) => parsed_time.format("%d/%m/%Y").to_string(),
				Err(_) => String::from("unknown"),
			}
		))
		.field("KZT", filters.0, true)
		.field("SKZ", filters.1, true)
		.field("VNL", filters.2, true)
		.footer(|f| {
			let icon_url = env::var("ICON").unwrap_or(String::from("unknown"));

			f.text("(͡ ͡° ͜ つ ͡͡°)7 | <3 to kzgo.eu").icon_url(icon_url)
		})
		.to_owned();

	SchnoseCommand::Embed(embed)
}
