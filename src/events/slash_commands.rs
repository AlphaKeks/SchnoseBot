use {
	std::{collections::HashMap, env},
	crate::{commands, schnose::BotData, db::UserSchema},
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

	let mut state = match GlobalState::new(&interaction, &ctx.http, &data.db) {
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

	match response {
		Err(why) => log::error!("Failed executing command: {:?}", why),
		Ok(response) => {
			if let Err(why) = state.reply(response).await {
				log::error!("Failed replying to interaction: {:?}", why);
			}
		},
	}

	return Ok(());
}

#[derive(Debug, Clone)]
pub(crate) struct GlobalState<'h> {
	http: &'h Http,
	interaction: &'h ApplicationCommandInteraction,
	pub deferred: bool,
	pub user: &'h User,
	pub opts: HashMap<String, json::Value>,
	pub db: &'h Collection<UserSchema>,
	pub req_client: reqwest::Client,
	pub colour: (u8, u8, u8),
	pub icon: String,
}

impl<'h> GlobalState<'h> {
	pub fn new(
		interaction: &'h ApplicationCommandInteraction,
		http: &'h Http,
		collection: &'h Collection<UserSchema>,
	) -> anyhow::Result<GlobalState<'h>> {
		let mut opts = HashMap::<String, json::Value>::new();
		for opt in &interaction.data.options {
			if let Some(value) = opt.value.to_owned() {
				opts.insert(opt.name.clone(), value);
			}
		}

		return Ok(Self {
			http,
			interaction,
			deferred: false,
			user: &interaction.user,
			opts,
			db: collection,
			req_client: reqwest::Client::new(),
			colour: (116, 128, 194),
			icon: env::var("ICON_URL").unwrap_or(
				String::from("https://cdn.discordapp.com/attachments/981130651094900756/981130719537545286/churchOfSchnose.png")
			)
		});
	}

	pub async fn defer(&mut self) -> anyhow::Result<()> {
		self.interaction.defer(self.http).await?;
		self.deferred = true;
		log::info!("deferred interaction `{}`", &self.interaction.data.name);
		return Ok(());
	}

	pub async fn reply(&self, content: InteractionResponseData) -> anyhow::Result<()> {
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

	pub fn get<T>(&self, name: &str) -> Option<T>
	where
		T: serde::de::DeserializeOwned,
	{
		match self.opts.get(name) {
			Some(value) => serde_json::from_value(value.clone()).ok(),
			None => None,
		}
	}

	pub fn thumbnail(&self, map_name: &String) -> String {
		return format!(
			"https://raw.githubusercontent.com/KZGlobalTeam/map-images/master/images/{}.jpg",
			map_name
		);
	}
}

#[derive(Debug, Clone)]
pub(crate) enum InteractionResponseData {
	Message(String),
	Embed(CreateEmbed),
}
