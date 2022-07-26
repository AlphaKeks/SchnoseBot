use {
	crate::{events::interactions::InteractionState, prelude::InteractionResult},
	gokz_rs::{
		prelude::*,
		global_api::{get_maps, is_global, get_filters},
		kzgo,
	},
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

pub(crate) async fn execute(state: &mut InteractionState<'_>) -> InteractionResult {
	// Defer current interaction since this could take a while
	state.defer().await?;

	let map_name = state.get::<String>("map_name").expect("This option is marked as `required`.");

	let (map_api, map_kzgo) = {
		let global_maps = get_maps(state.req_client).await?;
		let map_api = is_global(&MapIdentifier::Name(map_name), &global_maps).await?;
		let map_identifier = MapIdentifier::Name(map_api.name.clone());
		let map_kzgo = kzgo::maps::get_map(&map_identifier, state.req_client).await?;

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

	let filters = match get_filters(map_api.id, state.req_client).await {
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
			return Err(why.into());
		},
	};

	let embed = CreateEmbed::default()
		.color(state.colour)
		.title(&map_api.name)
		.url(state.map_link(&map_api.name))
		.thumbnail(&state.map_thumbnail(&map_api.name))
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

	Ok(embed.into())
}
