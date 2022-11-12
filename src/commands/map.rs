use {
	crate::events::slash_command::{
		InteractionData,
		InteractionResponseData::{Message, Embed},
	},
	anyhow::Result,
	gokz_rs::{prelude::*, global_api::*, kzgo},
	serenity::{
		builder::{CreateApplicationCommand, CreateEmbed},
		model::prelude::command::CommandOptionType,
	},
};

pub fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	return cmd.name("map").description("Get detailed information on a map.").create_option(
		|opt| {
			opt.kind(CommandOptionType::String)
				.name("map_name")
				.description("Specify a map.")
				.required(true)
		},
	);
}

pub async fn execute(mut ctx: InteractionData<'_>) -> Result<()> {
	ctx.defer().await?;

	let map_name = ctx.get_string("map_name").expect("This option is marked as `required`.");

	let client = reqwest::Client::new();
	let (map_api, map_kzgo) = {
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
				return ctx.reply(Message(&why.tldr)).await;
			},
		};

		let map_api = match is_global(&MapIdentifier::Name(map_name), &global_maps).await {
			Ok(map) => map,
			Err(why) => {
				log::warn!("[{}]: {} => {}\n{:?}", file!(), line!(), "Map is not global.", why);
				return ctx.reply(Message(&why.tldr)).await;
			},
		};

		let map_identifier = MapIdentifier::Name(map_api.name.clone());

		let map_kzgo = match kzgo::maps::get_map(&map_identifier, &client).await {
			Ok(map) => map,
			Err(why) => {
				log::error!(
					"[{}]: {} => {}\n{:?}",
					file!(),
					line!(),
					"Failed to fetch KZGO map.",
					why
				);
				return ctx.reply(Message(&why.tldr)).await;
			},
		};

		(map_api, map_kzgo)
	};

	let mappers = match map_kzgo.mapperNames {
		Some(names) => match map_kzgo.mapperIds {
			Some(ids) => names
				.into_iter()
				.enumerate()
				.map(|(idx, name)| {
					format!("[{}](https://steamcommunity.com/profiles/{})", name, ids[idx])
				})
				.collect::<Vec<String>>(),
			None => vec![String::new()],
		},
		None => vec![String::new()],
	};

	let filters = match get_filters(map_api.id, &client).await {
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
			log::error!(
				"[{}]: {} => {}\n{:?}",
				file!(),
				line!(),
				"Failed to fetch map filters",
				why
			);
			return ctx.reply(Message(&why.tldr)).await;
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
			// Discord's formatting sucks
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

	return ctx.reply(Embed(embed)).await;
}
