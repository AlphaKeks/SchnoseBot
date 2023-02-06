use {
	crate::{Context, GlobalMapsContainer, GLOBAL_MAPS},
	futures::StreamExt,
};

mod apistatus;
pub use apistatus::apistatus;

mod map;
pub use map::map;

mod ping;
pub use ping::ping;

async fn autocomplete_map<'a>(
	_ctx: Context<'_>, partial: &'a str,
) -> impl futures::Stream<Item = String> + 'a {
	loop {
		if let Ok(maps) = GLOBAL_MAPS.try_get() {
			break futures::stream::iter(maps).filter_map(move |map| async {
				if map
					.name
					.contains(&partial.to_lowercase())
				{
					Some(map.name.clone())
				} else {
					None
				}
			});
		} else {
			continue;
		}
	}
}
