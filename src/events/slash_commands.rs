use {
	std::collections::HashMap,
	crate::{
		commands,
		schnose::{BotData, SchnoseErr, InteractionResult},
		db::UserSchema,
	},
	serenity::{
		prelude::Context,
		model::{
			prelude::interaction::application_command::ApplicationCommandInteraction, user::User,
		},
		http::Http,
		json,
		builder::CreateEmbed,
	},
	mongodb::Collection,
};

pub(crate) async fn handle(
	data: &BotData,
	ctx: Context,
	interaction: ApplicationCommandInteraction,
) -> anyhow::Result<()> {
	let event_name = interaction.data.name.as_str();
	log::info!("received slash command: `{}`", event_name);

	// (almost) every command will receive some global state to access information about the
	// current interaction, the database etc.
	let mut state = match InteractionState::new(
		&interaction,
		&ctx.http,
		&data.db,
		&data.req_client,
		&data.icon,
	) {
		Ok(state) => {
			log::trace!("Created new interaction data.");
			state
		},
		Err(why) => {
			log::error!("Failed to create new interaction data.\n\n{:?}", why);
			return Err(why);
		},
	};

	let response = match event_name {
		"ping" => commands::ping::execute().await,
		"apistatus" => commands::apistatus::execute(&mut state).await,
		"bpb" => commands::bpb::execute(&mut state).await,
		"bwr" => commands::bwr::execute(&mut state).await,
		"db" => commands::db::execute(&mut state).await,
		"invite" => commands::invite::execute().await,
		"map" => commands::map::execute(&mut state).await,
		"mode" => commands::mode::execute(&mut state).await,
		"nocrouch" => commands::nocrouch::execute(&state).await,
		"pb" => commands::pb::execute(&mut state).await,
		"profile" => commands::profile::execute(&mut state).await,
		"random" => commands::random::execute(&state).await,
		"recent" => commands::recent::execute(&mut state).await,
		"setsteam" => commands::setsteam::execute(&mut state).await,
		"unfinished" => commands::unfinished::execute(&mut state).await,
		"wr" => commands::wr::execute(&mut state).await,
		unknown_command => {
			log::warn!("encountered unknown slash command: {}", unknown_command);
			return Ok(());
		},
	};

	if let Err(why) = state.reply(response).await {
		log::error!(
			"[{}]: {} => Something happened when replying to an interaction",
			file!(),
			line!()
		);
		log::error!("[{}]: {} => {:?}", file!(), line!(), why);
	};

	return Ok(());
}

/// Global State object holding information about the current interaction
/// -> will be passed to most commands
#[derive(Debug, Clone)]
pub(crate) struct InteractionState<'h> {
	http: &'h Http,
	// original Discord Interaction which created this instance
	interaction: &'h ApplicationCommandInteraction,
	// interaction options holding `{ name: value }` pairs for each command parameter passed by
	// the user
	opts: HashMap<String, json::Value>,
	// whether the current interaction has been deferred or not
	pub deferred: bool,
	// user who triggered this interaction
	pub user: &'h User,
	// reference to the database; gets passed from `BotData`
	pub db: &'h Collection<UserSchema>,
	// global reqwest Client to pass to `gokz_rs` functions that need it; gets passed from `BotData`
	pub req_client: &'h reqwest::Client,
	// #7480c2 -> only here so that I don't have to type it out over and over again
	pub colour: (u8, u8, u8),
	// icon to put into embed footers
	pub icon: &'h String,
}

impl<'h> InteractionState<'h> {
	pub fn new(
		interaction: &'h ApplicationCommandInteraction,
		http: &'h Http,
		collection: &'h Collection<UserSchema>,
		req_client: &'h reqwest::Client,
		icon: &'h String,
	) -> anyhow::Result<InteractionState<'h>> {
		let mut opts = HashMap::<String, json::Value>::new();

		// filter out the relevant information from the interaction data and put it into a HashMap
		for opt in &interaction.data.options {
			if let Some(value) = opt.value.to_owned() {
				opts.insert(opt.name.clone(), value);
			}
		}

		return Ok(Self {
			http,
			interaction,
			opts,
			deferred: false,
			user: &interaction.user,
			db: collection,
			req_client,
			colour: (116, 128, 194),
			icon,
		});
	}

	/// Wrapper function to defer the current interaction
	pub async fn defer(&mut self) -> Result<(), SchnoseErr> {
		if let Err(why) = self.interaction.defer(self.http).await {
			log::error!("[{}]: {} => {:?}", file!(), line!(), why);
			return Err(SchnoseErr::Defer);
		}
		self.deferred = true;
		log::info!("deferred interaction `{}`", &self.interaction.data.name);
		return Ok(());
	}

	/// Will be used to reply to an interaction, once the data for the reply has finished being
	/// generated
	async fn reply(&self, content: InteractionResult) -> anyhow::Result<()> {
		let content = match content {
			Ok(reply) => {
				log::trace!("Received successful interaction: {:?}", &reply);
				reply
			},
			Err(error) => {
				log::trace!("Received failed interaction: {:?}", &error);
				InteractionResponseData::Message(error.to_string())
			},
		};

		// Interaction has been deferred => edit original message
		if self.deferred {
			match self
				.interaction
				.edit_original_interaction_response(self.http, |response| match content {
					InteractionResponseData::Message(message) => response.content(message),
					InteractionResponseData::Embed(embed) => response.set_embed(embed),
				})
				.await
			{
				Ok(_) => log::info!("responded to interaction `{}`", &self.interaction.data.name),
				Err(why) => log::error!(
					"failed to respond to interaction `{}`\n\n{:?}",
					&self.interaction.data.name,
					why
				),
			}
		// Interaction has not been deferred => create a new message to reply with
		} else {
			match self
				.interaction
				.create_interaction_response(self.http, |response| {
					response.interaction_response_data(|response| match content {
						InteractionResponseData::Message(message) => response.content(message),
						InteractionResponseData::Embed(embed) => response.set_embed(embed),
					})
				})
				.await
			{
				Ok(_) => log::info!("responded to interaction `{}`", &self.interaction.data.name),
				Err(why) => log::error!(
					"failed to respond to interaction `{}`\n\n{:?}",
					&self.interaction.data.name,
					why
				),
			}
		}
		return Ok(());
	}

	/// Wrapper function to easily get a value from `self.opts` as a native type, instead of JSON
	pub fn get<T>(&self, name: &str) -> Option<T>
	where
		T: serde::de::DeserializeOwned,
	{
		match self.opts.get(name) {
			Some(value) => serde_json::from_value(value.clone()).ok(),
			None => None,
		}
	}

	/// Utility function to generate a map thumbnail URL so I don't have to type it over and over
	/// again
	pub fn thumbnail(&self, map_name: &str) -> String {
		return format!(
			"https://raw.githubusercontent.com/KZGlobalTeam/map-images/master/images/{}.jpg",
			map_name
		);
	}
}

/// Kind of data that can be generated by a slash command, so it can be sent to the user
#[derive(Debug, Clone)]
pub(crate) enum InteractionResponseData {
	Message(String),
	Embed(CreateEmbed),
}
