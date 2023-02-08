mod apistatus;
pub use apistatus::apistatus;

mod bmaptop;
pub use bmaptop::bmaptop;

mod bpb;
pub use bpb::bpb;

mod bwr;
pub use bwr::bwr;

mod db;
pub use db::db;

mod invite;
pub use invite::invite;

mod map;
pub use map::map;

mod maptop;
pub use maptop::maptop;

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

mod pagination {
	use {
		crate::{error::Error, Context},
		poise::serenity_prelude::{
			CollectComponentInteraction, CreateEmbed, InteractionResponseType,
		},
		std::time::Duration,
	};

	pub async fn paginate(ctx: &Context<'_>, embeds: Vec<CreateEmbed>) -> Result<(), Error> {
		let ctx_id = ctx.id();
		let prev_id = format!("{ctx_id}_prev");
		let next_id = format!("{ctx_id}_next");

		// Send initial reply
		ctx.send(|reply| {
			reply
				.embed(|e| {
					*e = embeds[0].clone();
					e
				})
				.components(|c| {
					c.create_action_row(|row| {
						row.create_button(|b| b.custom_id(&prev_id).label('◀'))
							.create_button(|b| b.custom_id(&next_id).label('▶'))
					})
				})
		})
		.await?;

		// Listen for button presses
		let mut current_page = 0;
		while let Some(interaction) = CollectComponentInteraction::new(ctx)
			.filter(move |press| {
				press
					.data
					.custom_id
					.starts_with(&ctx_id.to_string())
			})
			.timeout(Duration::from_secs(600))
			.await
		{
			if interaction.data.custom_id != prev_id && interaction.data.custom_id != next_id {
				continue;
			}

			if interaction.data.custom_id == prev_id {
				if current_page == 0 {
					current_page = embeds.len() - 1;
				} else {
					current_page -= 1;
				}
			} else {
				current_page += 1;
				if current_page >= embeds.len() {
					current_page = 0;
				}
			}

			interaction
				.create_interaction_response(ctx, |response| {
					response
						.kind(InteractionResponseType::UpdateMessage)
						.interaction_response_data(|data| {
							data.set_embed(embeds[current_page].clone())
						})
				})
				.await?;
		}

		Ok(())
	}
}
