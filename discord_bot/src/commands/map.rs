use {
	super::autocompletion::autocomplete_map,
	crate::{
		error::{Error, Result},
		Context, State,
	},
	gokz_rs::MapIdentifier,
};

/// Get detailed information on a map.
///
/// This command will fetch a bunch of useful information about a particular map. The information \
/// is a combination of the \
/// [GlobalAPI](https://kztimerglobal.com/swagger/index.html?urls.primaryName=V2), \
/// [n4vyn's](https://github.com/n4vyn) [KZ:GO API](https://kzgo.eu/) and my own \
/// [SchnoseAPI](https://github.com/AlphaKeks/SchnoseAPI). If anything seems incorrect, feel free \
/// to report it.
#[tracing::instrument(skip(ctx), fields(user = ctx.author().tag()))]
#[poise::command(slash_command, on_error = "Error::handle_command")]
pub async fn map(
	ctx: Context<'_>,

	#[autocomplete = "autocomplete_map"]
	#[rename = "map"]
	map_choice: String,
) -> Result<()> {
	ctx.defer().await?;

	let map = ctx.get_map(&MapIdentifier::Name(map_choice))?;

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
