mod apistatus;
pub use apistatus::apistatus;

mod bmaptop;
pub use bmaptop::bmaptop;

mod bpb;
pub use bpb::bpb;

mod btop;
pub use btop::btop;

mod bwr;
pub use bwr::bwr;

mod db;
pub use db::db;

mod help;
pub use help::help;

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

mod profile;
pub use profile::profile;

mod pull;
pub use pull::pull;

mod random;
pub use random::random;

mod recent;
pub use recent::recent;

mod recompile;
pub use recompile::recompile;

mod report;
pub use report::report;

mod restart;
pub use restart::restart;

mod setsteam;
pub use setsteam::setsteam;

mod top;
pub use top::top;

mod unfinished;
pub use unfinished::unfinished;

mod wr;
pub use wr::wr;

mod autocompletion {
	use {
		crate::{Context, State},
		fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher},
	};

	// Provides autocompletion for map names on certain commands using some fuzzy finding algorithm
	// I found on the interent. :)
	pub async fn autocomplete_map<'a>(
		ctx: Context<'a>,
		input: &'a str,
	) -> impl futures::Stream<Item = String> + 'a {
		let fzf = SkimMatcherV2::default();
		let input = input.to_lowercase();
		let mut map_names = ctx
			.global_map_names()
			.iter()
			.filter_map(move |name| {
				let score = fzf.fuzzy_match(name, &input)?;
				if score > 50 || input.is_empty() {
					return Some((score, String::from(*name)));
				}
				None
			})
			.collect::<Vec<_>>();

		map_names.sort_unstable_by(|(a_score, _), (b_score, _)| b_score.cmp(a_score));

		futures::stream::iter(
			map_names
				.into_iter()
				.map(|(_, map_name)| map_name),
		)
	}
}

mod choices {
	use {
		crate::{
			db,
			error::{Error, Result},
		},
		gokz_rs::{Mode, Tier},
		poise::ChoiceParameter,
	};

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

	impl ModeChoice {
		pub fn parse_input(choice: Option<Self>, db_entry: &Result<db::User>) -> Result<Mode> {
			if let Some(mode) = choice {
				Ok(mode.into())
			} else {
				db_entry
					.as_ref()
					.map_err(|_| Error::MissingMode)?
					.mode
					.ok_or(Error::MissingMode)
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
		type Error = Error;

		fn try_from(value: DBModeChoice) -> Result<Self> {
			match value {
				DBModeChoice::None => Err(Error::MissingMode),
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

	#[derive(Debug, Clone, Copy, ChoiceParameter)]
	#[allow(clippy::upper_case_acronyms)]
	pub enum TierChoice {
		#[name = "VeryEasy"]
		VeryEasy = 1,

		#[name = "Easy"]
		Easy = 2,

		#[name = "Medium"]
		Medium = 3,

		#[name = "Hard"]
		Hard = 4,

		#[name = "VeryHard"]
		VeryHard = 5,

		#[name = "Extreme"]
		Extreme = 6,

		#[name = "Death"]
		Death = 7,
	}

	impl From<TierChoice> for Tier {
		fn from(value: TierChoice) -> Self {
			match value {
				TierChoice::VeryEasy => Self::VeryEasy,
				TierChoice::Easy => Self::Easy,
				TierChoice::Medium => Self::Medium,
				TierChoice::Hard => Self::Hard,
				TierChoice::VeryHard => Self::VeryHard,
				TierChoice::Extreme => Self::Extreme,
				TierChoice::Death => Self::Death,
			}
		}
	}
}

mod pagination {
	use {
		crate::{error::Result, Context},
		poise::serenity_prelude::{
			CollectComponentInteraction, CreateEmbed, InteractionResponseType,
		},
		std::time::Duration,
	};

	pub async fn paginate(ctx: &Context<'_>, embeds: Vec<CreateEmbed>) -> Result<()> {
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
