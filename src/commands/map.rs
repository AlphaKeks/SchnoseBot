use {
	crate::events::slash_commands::{
		InteractionData,
		InteractionResponseData::{Message, Embed},
	},
	gokz_rs::{prelude::*, global_api::*, kzgo},
	serenity::{
		builder::{CreateApplicationCommand, CreateEmbed},
		model::prelude::command::CommandOptionType,
	},
};

pub(crate) fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	return cmd.name("map").description("Get detailed information on a map.").create_option(
		|opt| {
			opt.kind(CommandOptionType::String)
				.name("map_name")
				.description("Specify a map.")
				.required(true)
		},
	);
}

pub(crate) async fn execute(mut data: InteractionData<'_>) -> anyhow::Result<()> {
	data.defer().await?;

	let map_name = data.get_string("map_name").expect("This option is marked as `required`.");

	let (map_api, map_kzgo) = {
		let global_maps = match get_maps(&data.req_client).await {
			Ok(maps) => maps,
			Err(why) => {
				log::warn!("[{}]: {} => {:?}", file!(), line!(), why);
				return data.reply(Message(&why.tldr)).await;
			},
		};

		let map_api = match is_global(&MapIdentifier::Name(map_name), &global_maps).await {
			Ok(map) => map,
			Err(why) => {
				log::warn!("[{}]: {} => {:?}", file!(), line!(), why);
				return data.reply(Message(&why.tldr)).await;
			},
		};

		let map_identifier = MapIdentifier::Name(map_api.name.clone());

		let map_kzgo = match kzgo::maps::get_map(&map_identifier, &data.req_client).await {
			Ok(map) => map,
			Err(why) => {
				log::warn!("[{}]: {} => {:?}", file!(), line!(), why);
				return data.reply(Message(&why.tldr)).await;
			},
		};

		(map_api, map_kzgo)
	};

	let mappers = {
		let mut mappers: Vec<String> = Vec::new();
		if let Some(names) = map_kzgo.mapperNames {
			if let Some(ids) = map_kzgo.mapperIds {
				for (i, name) in names.into_iter().enumerate() {
					mappers.push(format!(
						"[{}](https://steamcommunity.com/profiles/{})",
						name, ids[i]
					));
				}
			}
		}
		mappers
	};

	let filters = match get_filters(map_api.id, &data.req_client).await {
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
		Err(why) => {
			log::warn!("[{}]: {} => {:?}", file!(), line!(), why);
			return data.reply(Message(&why.tldr)).await;
		},
	};

	let embed = CreateEmbed::default()
		.color(data.colour)
		.title(&map_api.name)
		.url(format!("https://kzgo.eu/maps/{}", &map_api.name))
		.thumbnail(&data.thumbnail(&map_api.name))
		.description(format!(
			r#"
🢂 API Tier: {}
🢂 Mapper(s): {}
🢂 Bonuses: {}
🢂 Last Update: {}

🢂 Filters:
			"#,
			&map_api.difficulty,
			mappers.join(", "),
			&map_kzgo.bonuses.unwrap_or(0),
			match chrono::NaiveDateTime::parse_from_str(&map_api.updated_on, "%Y-%m-%dT%H:%M:%S") {
				Ok(parsed_time) => parsed_time.format("%d/%m/%Y").to_string(),
				Err(_) => String::from("unknown"),
			},
		))
		.field("KZT", filters.0, true)
		.field("SKZ", filters.1, true)
		.field("VNL", filters.2, true)
		.to_owned();

	return data.reply(Embed(embed)).await;
}
