use {
	super::autocompletion::autocomplete_map,
	crate::{error::Error, Context, GlobalMapsContainer, State, GLOBAL_MAPS},
	gokz_rs::{prelude::*, GlobalAPI},
	log::trace,
};

#[poise::command(slash_command, on_error = "Error::handle_command")]
pub async fn map(
	ctx: Context<'_>, #[autocomplete = "autocomplete_map"] map_name: String,
) -> Result<(), Error> {
	ctx.defer().await?;
	trace!("[/map] map_name: `{map_name}`");

	let map = GLOBAL_MAPS.find(&MapIdentifier::Name(map_name))?;

	let mappers = map
		.mapper_names
		.iter()
		.zip(map.mapper_ids)
		.fold(Vec::new(), |mut names, (name, id)| {
			names.push(format!("[{name}](https://steamcommunity.com/profiles/{id})"));
			names
		});

	let (kzt_filter, skz_filter, vnl_filter) =
		GlobalAPI::get_filters(map.id as i32, ctx.gokz_client())
			.await?
			.into_iter()
			.filter(|f| f.stage == 0)
			.fold(("❌", "❌", "❌"), |mut symbols, f| {
				match f.mode_id {
					200 => symbols.0 = "✅",
					201 => symbols.1 = "✅",
					202 => symbols.2 = "✅",
					_ => {}
				}
				symbols
			});

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
					&map.courses,
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
