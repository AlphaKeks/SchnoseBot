use gokz_rs::{
	global_api::{get_filters, get_maps, is_global},
	kzgo,
	prelude::MapIdentifier,
};
use serenity::{
	builder::{CreateApplicationCommand, CreateEmbed},
	model::prelude::command::CommandOptionType,
};

use crate::event_handler::interaction_create::{Metadata, SchnoseResponseData};

pub fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	cmd.name("map")
		.description("Get detailed information on a map.")
		.create_option(|opt| {
			opt.kind(CommandOptionType::String)
				.name("map_name")
				.description("Specify a map.")
				.required(true)
		})
}

pub async fn run(metadata: Metadata) {
	let client = reqwest::Client::new();

	// sanitaze user input
	let (map_api, map_kzgo) = match metadata.opts.get_string("map_name") {
		Some(map_name) => {
			let global_maps = match get_maps(&client).await {
				Err(why) => {
					log::error!(
						"[{}]: {} => {}\n{:#?}",
						file!(),
						line!(),
						"Failed to get global maps",
						why
					);
					return metadata.reply(SchnoseResponseData::Message(why.tldr)).await;
				},
				Ok(maps) => maps,
			};

			let map_identifier = MapIdentifier::Name(map_name);

			let map = match is_global(&map_identifier, &global_maps).await {
				Err(why) => {
					log::error!(
						"[{}]: {} => {}\n{:#?}",
						file!(),
						line!(),
						"Failed to get global maps",
						why
					);
					return metadata.reply(SchnoseResponseData::Message(why.tldr)).await;
				},
				Ok(map) => map,
			};

			let kzgo_map =
				match kzgo::maps::get_map(&MapIdentifier::Name(map.name.clone()), &client).await {
					Err(why) => {
						log::error!(
							"[{}]: {} => {}\n{:#?}",
							file!(),
							line!(),
							"Failed to get global maps",
							why
						);
						return metadata.reply(SchnoseResponseData::Message(why.tldr)).await;
					},
					Ok(map) => map,
				};

			(map, kzgo_map)
		},
		None => unreachable!("option is required"),
	};

	let mappers = match map_kzgo.mapperNames {
		Some(names) => match map_kzgo.mapperIds {
			Some(ids) => names
				.into_iter()
				.enumerate()
				.map(|(idx, name)| {
					format!("[{}](https://steamcommuntiy.com/profiles/{})", name, ids[idx])
				})
				.collect::<Vec<String>>(),
			None => todo!(),
		},
		None => todo!(),
	};

	let filters = match get_filters(map_api.id, &client).await {
		Err(why) => {
			log::error!("[{}]: {} => {}\n{:#?}", file!(), line!(), "Failed to get filters.", why);
			return metadata.reply(SchnoseResponseData::Message(why.tldr)).await;
		},
		Ok(filters) => {
			let filters = filters.into_iter().filter(|f| f.stage == 0).collect::<Vec<_>>();
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
		},
	};

	let embed = CreateEmbed::default()
		.color((116, 128, 194))
		.title(&map_api.name)
		.url(format!("https://kzgo.eu/maps/{}", &map_api.name))
		.thumbnail(format!(
			"https://raw.githubusercontent.com/KZGlobalTeam/map-images/master/images/{}.jpg",
			&map_api.name
		))
		.description(format!(
			// this is super ugly but Discord's formatting on mobile sucks
			"
🢂 API Tier: {}
🢂 Mapper(s): {}
🢂 Bonuses: {}
🢂 Last Update: {}

🢂 Filters:
			",
			&map_api.difficulty,
			mappers.join(", "),
			match &map_kzgo.bonuses {
				Some(n) => n,
				None => &0,
			},
			match chrono::NaiveDateTime::parse_from_str(&map_api.updated_on, "%Y-%m-%dT%H:%M:%S") {
				Ok(parsed_time) => parsed_time.format("%d/%m/%Y").to_string(),
				Err(_) => String::from("unknown"),
			}
		))
		.field("KZT", filters.0, true)
		.field("SKZ", filters.1, true)
		.field("VNL", filters.2, true)
		.to_owned();

	return metadata.reply(SchnoseResponseData::Embed(embed)).await;
}
