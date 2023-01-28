use {
	crate::{
		GlobalState, formatting,
		discord::Mention,
		database,
		SchnoseError::{self, *},
	},
	std::time::Duration,
	log::{debug, info, error},
	gokz_rs::prelude::*,
	futures::StreamExt,
	lazy_static::lazy_static,
	poise::serenity_prelude::{CreateEmbed, CollectComponentInteraction},
	sqlx::MySql,
};

pub mod apistatus;
pub mod bmaptop;
pub mod bpb;
pub mod btop;
pub mod bwr;
pub mod db;
pub mod help;
pub mod invite;
pub mod map;
pub mod maptop;
pub mod mode;
pub mod nocrouch;
pub mod pb;
pub mod ping;
pub mod profile;
pub mod pull;
pub mod random;
pub mod recent;
pub mod recompile;
pub mod report;
pub mod restart;
pub mod setsteam;
pub mod top;
pub mod unfinished;
pub mod update;
pub mod wr;

async fn handle_err(error: poise::FrameworkError<'_, GlobalState, crate::SchnoseError>) {
	use poise::FrameworkError::*;

	error!("Failed to handle slash command.");

	let (content, ephemeral) = match &error {
		Command { error, ctx: _ } => (error.to_string(), false),
		ArgumentParse { error: _, input: _, ctx: _ } => {
			(String::from("Failed to parse arguments."), false)
		},
		CommandStructureMismatch { description, ctx: _ } => {
			error!("{}", description);
			(String::from("Incorrect command structure."), false)
		},
		CooldownHit { remaining_cooldown, ctx: _ } => (
			format!(
			"This command is currently on cooldown. Please wait another {} seconds before trying again.",
			remaining_cooldown.as_secs()
		    ),
			true,
		),
		MissingBotPermissions { missing_permissions, ctx: _ } => {
			error!("{}", missing_permissions);
			(
			    String::from("The bot doesn't have the required permissions in this channel/server to handle this command."),
			    false,
			)
		},
		MissingUserPermissions { missing_permissions, ctx: _ } => {
			error!("{:?}", missing_permissions);
			(
				String::from("You don't have the required permissions to execute this command."),
				true,
			)
		},
		NotAnOwner { ctx: _ } => return,
		UnknownCommand {
			ctx: _,
			msg: _,
			prefix,
			msg_content,
			framework: _,
			invocation_data: _,
			trigger,
		} => {
			error!("Prefix: {}", prefix);
			error!("Message: {}", msg_content);
			error!("Trigger: {:?}", trigger);
			(String::from("Unknown / Outdated command."), false)
		},
		UnknownInteraction { ctx: _, framework: _, interaction } => {
			error!("Interaction: {:?}", interaction);
			(String::from("Unknown / Outdated Interaction."), false)
		},
		_ => unreachable!(),
	};

	if let Some(ctx) = error.ctx() {
		if let Err(why) = ctx
			.send(|reply| {
				reply
					.ephemeral(ephemeral)
					.content(&content)
			})
			.await
		{
			error!("Failed to reply after failed slash command: {:?}", why);
		}

		error!("Handled error with `{}`.", content);
	}
}

trait Page {
	fn to_field(&self, i: usize) -> (String, String, bool);
}

impl Page for gokz_rs::records::Record {
	fn to_field(&self, i: usize) -> (String, String, bool) {
		(
			format!(
				"{} [#{}]",
				self.player_name
					.as_ref()
					.map_or(String::from("unknown"), |name| name.to_owned()),
				i
			),
			format!(
				"{}{}",
				formatting::format_time(self.time),
				if self.teleports > 0 {
					format!(" ({} TPs)", self.teleports)
				} else {
					String::new()
				}
			),
			true,
		)
	}
}

async fn paginate<F, P>(
	elements: Vec<P>,
	get_embed: F,
	timeout: Duration,
	ctx: &crate::Context<'_>,
) -> Result<(), crate::SchnoseError>
where
	F: Fn(usize, usize) -> CreateEmbed,
	P: Page,
{
	let mut embeds = Vec::new();
	let len = elements.len();

	let mut temp_embed = get_embed(1, len);
	for (i, element) in elements.into_iter().enumerate() {
		// We have reached 12 fields on `temp_embed`. Push it to the embed list and reset it.
		if i >= 12 && i % 12 == 0 {
			embeds.push(temp_embed);
			temp_embed = get_embed(embeds.len() + 1, len);
		}

		let (name, value, inline) = element.to_field(i + 1);
		temp_embed.field(name, value, inline);

		// We only have 1 page -> push final element and break
		if i + 1 == len {
			embeds.push(temp_embed);
			break;
		}
	}

	let ctx_id = ctx.id();
	let prev_id = format!("{}_prev", ctx.id());
	let next_id = format!("{}_next", ctx.id());

	// Create initial response
	ctx.send(|reply| {
		reply
			.embed(|e| {
				*e = embeds[0].clone();
				e
			})
			.components(|components| {
				components.create_action_row(|row| {
					row.create_button(|b| b.custom_id(&prev_id).label('◀'))
						.create_button(|b| b.custom_id(&next_id).label('▶'))
				})
			})
	})
	.await?;

	// Loop over incoming interactions of the buttons
	let mut current_page = 0;
	while let Some(press) = CollectComponentInteraction::new(ctx)
		// We only want to handle interactions belonging to the current message
		.filter(move |press| {
			press
				.data
				.custom_id
				.starts_with(&ctx_id.to_string())
		})
		// Listen for 10 minutes
		.timeout(timeout)
		.await
	{
		if press.data.custom_id != prev_id && press.data.custom_id != next_id {
			// irrelevant interaction
			continue;
		}

		if press.data.custom_id == prev_id {
			if current_page == 0 {
				current_page = embeds.len() - 1;
			} else {
				current_page -= 1;
			}
		} else if press.data.custom_id == next_id {
			current_page += 1;
			if current_page >= embeds.len() {
				current_page = 0;
			}
		}

		// Update message with new page
		press
			.create_interaction_response(ctx, |response| {
				response
					.kind(poise::serenity_prelude::InteractionResponseType::UpdateMessage)
					.interaction_response_data(|data| data.set_embed(embeds[current_page].clone()))
			})
			.await?
	}

	Ok(())
}

lazy_static! {
	/// Cached version of the global map pool
	static ref GLOBAL_MAPS: Vec<gokz_rs::maps::Map> =
		serde_json::from_str(include_str!(concat!(env!("OUT_DIR"), "/global_maps.json")))
			.expect("Failed to parse cached global maps.");

	static ref MAP_NAMES: Vec<String> = (*GLOBAL_MAPS).iter().map(|h| h.name.clone()).collect();
}

#[allow(dead_code)]
async fn autocomplete_map<'a>(
	_ctx: crate::Context<'_>,
	partial: &'a str,
) -> impl futures::Stream<Item = &'a String> + 'a {
	futures::stream::iter(&*MAP_NAMES)
		.filter(|name| futures::future::ready(name.contains(&partial.to_lowercase())))
}

#[derive(Debug, Clone, Copy, poise::ChoiceParameter)]
pub enum ModeChoice {
	#[name = "KZTimer"]
	KZTimer = 200,
	#[name = "SimpleKZ"]
	SimpleKZ = 201,
	#[name = "Vanilla"]
	Vanilla = 202,
}

async fn mode_from_choice(
	choice: &Option<ModeChoice>,
	target: &Target,
	pool: &sqlx::Pool<MySql>,
) -> Result<Mode, SchnoseError> {
	match choice {
		Some(ModeChoice::KZTimer) => Ok(Mode::KZTimer),
		Some(ModeChoice::SimpleKZ) => Ok(Mode::SimpleKZ),
		Some(ModeChoice::Vanilla) => Ok(Mode::Vanilla),
		_ => target.get_mode(pool).await,
	}
}

#[derive(Debug, Clone, Copy, poise::ChoiceParameter)]
pub enum DBModeChoice {
	#[name = "KZTimer"]
	KZTimer = 200,
	#[name = "SimpleKZ"]
	SimpleKZ = 201,
	#[name = "Vanilla"]
	Vanilla = 202,
	#[name = "None"]
	None = 0,
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy, poise::ChoiceParameter)]
pub enum RuntypeChoice {
	#[name = "TP"]
	TP,
	#[name = "PRO"]
	PRO,
}

#[derive(Debug, Clone, Copy, poise::ChoiceParameter)]
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
			TierChoice::VeryEasy => Tier::VeryEasy,
			TierChoice::Easy => Tier::Easy,
			TierChoice::Medium => Tier::Medium,
			TierChoice::Hard => Tier::Hard,
			TierChoice::VeryHard => Tier::VeryHard,
			TierChoice::Extreme => Tier::Extreme,
			TierChoice::Death => Tier::Death,
		}
	}
}

#[derive(Debug, Clone)]
pub enum Target {
	None(u64),
	Mention(u64),
	SteamID(SteamID),
	PlayerName(String),
}

impl Target {
	pub async fn query_db(
		&self,
		pool: &sqlx::Pool<MySql>,
		filter: &str,
	) -> Result<database::UserSchema, SchnoseError> {
		info!("Querying database...");

		let query = format!("SELECT * FROM discord_users WHERE {filter}");

		debug!("query: {}", query);

		let query = sqlx::query_as::<_, database::UserSchema>(&query)
			.fetch_one(pool)
			.await?;

		info!("Finished querying database.");

		Ok(query)
	}

	pub async fn get_steam_id(&self, pool: &sqlx::Pool<MySql>) -> Result<SteamID, SchnoseError> {
		let filter = match self {
			Target::None(user_id) | Target::Mention(user_id) => format!("discord_id = {user_id}"),
			Target::SteamID(steam_id) => format!(r#"steam_id = "{steam_id}""#),
			Target::PlayerName(name) => format!(r#"name = "{name}""#),
		};

		let query = self.query_db(pool, &filter).await?;

		match query.steam_id {
			Some(steam_id) => Ok(steam_id.parse()?),
			None => Err(NoSteamID { blame_user: matches!(self, Target::None(_)) }),
		}
	}

	pub async fn get_mode(&self, pool: &sqlx::Pool<MySql>) -> Result<Mode, SchnoseError> {
		let filter = match self {
			Target::None(user_id) | Target::Mention(user_id) => format!("discord_id = {user_id}"),
			Target::SteamID(steam_id) => format!(r#"steam_id = "{steam_id}""#),
			Target::PlayerName(name) => format!(r#"name = "{name}""#),
		};

		let query = self.query_db(pool, &filter).await?;

		match query.mode {
			Some(mode_id) => Ok(Mode::try_from(mode_id)?),
			None => Err(NoMode),
		}
	}

	fn from_input(value: Option<String>, user_id: u64) -> Self {
		let Some(value) = value else {
			return Self::None(user_id);
		};

		if let Ok(mention) = value.parse::<Mention>() {
			return Self::Mention(*mention);
		}

		if let Ok(steam_id) = SteamID::new(&value) {
			return Self::SteamID(steam_id);
		}

		Self::PlayerName(value)
	}

	async fn to_player(&self, pool: &sqlx::Pool<MySql>) -> Result<PlayerIdentifier, SchnoseError> {
		match self {
			Target::None(_) | Target::Mention(_) => {
				let steam_id = self.get_steam_id(pool).await?;
				Ok(PlayerIdentifier::SteamID(steam_id))
			},
			Target::SteamID(steam_id) => Ok(PlayerIdentifier::SteamID(steam_id.to_owned())),
			Target::PlayerName(player_name) => Ok(PlayerIdentifier::Name(player_name.to_owned())),
		}
	}
}
