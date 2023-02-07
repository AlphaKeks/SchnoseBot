mod apistatus;
pub use apistatus::apistatus;

mod db;
pub use db::db;

mod invite;
pub use invite::invite;

mod map;
pub use map::map;

mod mode;
pub use mode::mode;

mod nocrouch;
pub use nocrouch::nocrouch;

mod pb;
pub use pb::pb;

mod ping;
pub use ping::ping;

mod recent;
pub use recent::recent;

mod report;
pub use report::report;

mod setsteam;
pub use setsteam::setsteam;

mod wr;
pub use wr::wr;

mod autocompletion {
	use {
		crate::{Context, GlobalMapsContainer, GLOBAL_MAPS},
		futures::StreamExt,
	};

	/// Provides autocompletion for map names on certain commands using the
	pub async fn autocomplete_map<'a>(
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
}

mod choices {
	use {crate::error, gokz_rs::prelude::*, poise::ChoiceParameter};

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
	pub enum DBModeChoice {
		#[name = "None"]
		None = 0,
		#[name = "KZTimer"]
		KZTimer = 200,
		#[name = "SimpleKZ"]
		SimpleKZ = 201,
		#[name = "Vanilla"]
		Vanilla = 202,
	}

	impl TryFrom<DBModeChoice> for Mode {
		type Error = error::Error;

		fn try_from(value: DBModeChoice) -> Result<Self, Self::Error> {
			match value {
				DBModeChoice::None => Err(error::Error::MissingMode),
				DBModeChoice::KZTimer => Ok(Self::KZTimer),
				DBModeChoice::SimpleKZ => Ok(Self::SimpleKZ),
				DBModeChoice::Vanilla => Ok(Self::Vanilla),
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

	#[derive(Debug, Clone, Copy, ChoiceParameter)]
	#[allow(clippy::upper_case_acronyms)]
	pub enum BoolChoice {
		#[name = "Yes"]
		Yes = 1,
		#[name = "No"]
		No = 0,
	}

	impl From<BoolChoice> for bool {
		fn from(value: BoolChoice) -> Self {
			matches!(value, BoolChoice::Yes)
		}
	}
}

mod pagination {}
