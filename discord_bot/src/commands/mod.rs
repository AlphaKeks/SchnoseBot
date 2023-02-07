use {
	crate::{Context, GlobalMapsContainer, GLOBAL_MAPS},
	futures::StreamExt,
	gokz_rs::prelude::*,
	poise::ChoiceParameter,
};

mod apistatus;
pub use apistatus::apistatus;

mod map;
pub use map::map;

mod pb;
pub use pb::pb;

mod ping;
pub use ping::ping;

mod wr;
pub use wr::wr;

/// Provides autocompletion for map names on certain commands using the
async fn autocomplete_map<'a>(
	_: Context<'a>, input: &'a str,
) -> impl futures::Stream<Item = String> + 'a {
	loop {
		if let Ok(maps) = GLOBAL_MAPS.try_get() {
			break futures::stream::iter(maps).filter_map(move |map| async {
				if map.name.contains(&input.to_lowercase()) {
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

#[derive(Debug, Clone, Copy, ChoiceParameter)]
pub enum ModeChoice {
	#[name = "KZTimer"]
	KZTimer = 200,
	#[name = "SimpleKZ"]
	SimpleKZ = 201,
	#[name = "Vanilla"]
	Vanilla = 202,
}

impl From<ModeChoice> for Mode {
	fn from(value: ModeChoice) -> Self {
		match value {
			ModeChoice::KZTimer => Self::KZTimer,
			ModeChoice::SimpleKZ => Self::SimpleKZ,
			ModeChoice::Vanilla => Self::Vanilla,
		}
	}
}

#[derive(Debug, Clone, Copy, ChoiceParameter)]
#[allow(clippy::upper_case_acronyms)]
pub enum RuntypeChoice {
	#[name = "TP"]
	TP = 1,
	#[name = "PRO"]
	PRO = 0,
}

impl From<RuntypeChoice> for bool {
	fn from(value: RuntypeChoice) -> Self {
		matches!(value, RuntypeChoice::TP)
	}
}
