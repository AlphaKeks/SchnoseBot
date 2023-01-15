use {
	super::{MAP_NAMES, autocomplete_map, handle_err},
	crate::{
		GlobalStateAccess, formatting,
		SchnoseError::{self, *},
	},
	log::trace,
	gokz_rs::{prelude::*, GlobalAPI, KZGO},
};

/// Will fetch detailed information about a given map.
#[poise::command(slash_command, on_error = "handle_err")]
pub async fn map(
	ctx: crate::Context<'_>,
	#[autocomplete = "autocomplete_map"] map_name: String,
) -> Result<(), SchnoseError> {
	ctx.defer().await?;

	trace!("[/map] map_name: `{}`", &map_name);

	let Some(map_name) = (*MAP_NAMES).iter().find(|name| name.contains(&map_name.to_lowercase())) else {
		return Err(InvalidMapName(map_name));
	};

	let map_identifier = MapIdentifier::Name(map_name.to_owned());

	let map_api = GlobalAPI::get_map(&map_identifier, ctx.gokz_client()).await?;
	let map_kzgo = KZGO::get_map(map_name, ctx.gokz_client()).await?;

	let mappers = if let Some((names, ids)) = map_kzgo
		.mapperNames
		.zip(map_kzgo.mapperIds)
	{
		names
			.iter()
			.zip(ids)
			.fold(Vec::new(), |mut names, (name, id)| {
				names.push(format!("[{}](https://steamcommunity.com/profiles/{})", name, id));
				names
			})
	} else {
		vec![String::from("unknown")]
	};

	let (kzt_filter, skz_filter, vnl_filter) =
		GlobalAPI::get_filters(map_api.id, ctx.gokz_client())
			.await?
			.into_iter()
			.filter(|f| f.stage == 0)
			.fold(("❌", "❌", "❌"), |mut symbols, f| {
				match f.mode_id {
					200 => symbols.0 = "✅",
					201 => symbols.1 = "✅",
					202 => symbols.2 = "✅",
					_ => {},
				}
				symbols
			});

	ctx.send(|reply| {
		reply.embed(|e| {
			e.color((116, 128, 194))
				.title(map_name)
				.url(formatting::map_link(map_name))
				.thumbnail(formatting::map_thumbnail(map_name))
				.description(format!(
					"
🢂 API Tier: {}
🢂 Mapper(s): {}
🢂 Bonuses: {}
🢂 Last Updated: {}

🢂 Filters:
				",
					&map_api.difficulty,
					mappers.join(", "),
					&map_kzgo.bonuses.unwrap_or(0),
					match chrono::NaiveDateTime::parse_from_str(
						&map_api.updated_on,
						"%Y-%m-%dT%H:%M:%S"
					) {
						Ok(parsed_time) => parsed_time
							.format("%d/%m/%Y")
							.to_string(),
						Err(_) => String::from("unknown"),
					},
				))
				.field("KZT", kzt_filter, true)
				.field("SKZ", skz_filter, true)
				.field("VNL", vnl_filter, true)
		})
	})
	.await?;

	Ok(())
}
