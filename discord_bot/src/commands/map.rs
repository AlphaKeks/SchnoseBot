use {
	super::autocompletion::autocomplete_map,
	crate::{
		error::{Error, Result},
		Context, State,
	},
	gokz_rs::MapIdentifier,
	log::trace,
};

/// Get detailed information on a map.
///
/// This includes the map's tier, mapper and filters. If you find any of this information to be
/// incorrect, feel free to report it.
#[poise::command(slash_command, on_error = "Error::handle_command")]
pub async fn map(
	ctx: Context<'_>,
	#[autocomplete = "autocomplete_map"] map_name: String,
) -> Result<()> {
	trace!("[/map ({})]", ctx.author().tag());
	trace!("> `map_name`: {map_name:?}");
	ctx.defer().await?;

	let map = ctx.get_map(&MapIdentifier::Name(map_name))?;

	let mapper = if let Some(steam_id) = map.mapper_steam_id {
		format!("[{}](https://steamcommunity.com/profiles/{})", map.mapper_name, steam_id.as_id64())
	} else {
		map.mapper_name
	};

	let kzt_filter = if map.kzt { "✅" } else { "❌" };
	let skz_filter = if map.skz { "✅" } else { "❌" };
	let vnl_filter = if map.vnl { "✅" } else { "❌" };

	ctx.send(|reply| {
		reply.embed(|e| {
			e.color(ctx.color())
				.title(&map.name)
				.url(&map.url)
				.thumbnail(&map.thumbnail)
				.description(format!(
					"
🢂 Tier: {} ({})
🢂 Mapper(s): {}
🢂 Bonuses: {}
🢂 Last Updated: {}

🢂 Filters:
				",
					map.tier as u8,
					map.tier,
					mapper,
					map.courses.len() - 1,
					map.updated_on.format("%d/%m/%Y")
				))
				.field("KZT", kzt_filter, true)
				.field("SKZ", skz_filter, true)
				.field("VNL", vnl_filter, true)
		})
	})
	.await?;

	Ok(())
}
