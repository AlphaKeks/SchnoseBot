use {
	super::autocompletion::autocomplete_map,
	crate::{
		error::{Error, Result},
		Context, State,
	},
	gokz_rs::prelude::*,
	log::trace,
};

/// Get detailed information on a map.
///
/// This command will combine information from both the \
/// [GlobalAPI](https://portal.global-api.com/dashboard) and [KZ:GO](https://kzgo.eu) to give you \
/// a compact summary of a given map's most important information.
#[poise::command(slash_command, on_error = "Error::handle_command")]
pub async fn map(
	ctx: Context<'_>,
	#[autocomplete = "autocomplete_map"] map_name: String,
) -> Result<()> {
	trace!("[/map ({})]", ctx.author().tag());
	trace!("> `map_name`: {map_name:?}");
	ctx.defer().await?;

	let map = ctx.get_map(&MapIdentifier::Name(map_name))?;

	let mappers = map
		.mapper_names
		.iter()
		.zip(map.mapper_ids)
		.fold(Vec::new(), |mut names, (name, id)| {
			names.push(format!("[{name}](https://steamcommunity.com/profiles/{id})"));
			names
		});

	let kzt_filter = if map.courses[0].kzt { "✅" } else { "❌" };
	let skz_filter = if map.courses[0].skz { "✅" } else { "❌" };
	let vnl_filter = if map.courses[0].vnl { "✅" } else { "❌" };

	ctx.send(|reply| {
		reply.embed(|e| {
			e.color(ctx.color())
				.title(&map.name)
				.url(&map.url)
				.thumbnail(&map.thumbnail)
				.description(format!(
					"
🢂 API Tier: {}
🢂 Mapper(s): {}
🢂 Bonuses: {}
🢂 Last Updated: {}

🢂 Filters:
				",
					&map.tier,
					mappers.join(", "),
					&map.courses.len(),
					&map.updated_on
						.format("%d/%m/%Y")
						.to_string()
				))
				.field("KZT", kzt_filter, true)
				.field("SKZ", skz_filter, true)
				.field("VNL", vnl_filter, true)
				.footer(|f| {
					f.text("<3 to kzgo.eu")
						.icon_url(ctx.icon())
				})
		})
	})
	.await?;

	Ok(())
}
