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

	let state = match GlobalState::new(&interaction, &ctx.http, &data.db) {
		Ok(state) => {
			log::trace!("Created new interaction data.");
			state
		},
		Err(why) => {
			log::error!("Failed to create new interaction data.\n\n{:?}", why);
			return Err(why);
		},
	};

	match event_name {
		"ping" => commands::ping::execute(state).await,
		"apistatus" => commands::apistatus::execute(state).await,
		"bpb" => commands::bpb::execute(state).await,
		"bwr" => commands::bwr::execute(state).await,
		"db" => commands::db::execute(state).await,
		"invite" => commands::invite::execute(state).await,
		"map" => commands::map::execute(state).await,
		"mode" => commands::mode::execute(state).await,
		"nocrouch" => commands::nocrouch::execute(state).await,
		"pb" => commands::pb::execute(state).await,
		"profile" => commands::profile::execute(state).await,
		"random" => commands::random::execute(state).await,
		"recent" => commands::recent::execute(state).await,
		"setsteam" => commands::setsteam::execute(state).await,
		"unfinished" => commands::unfinished::execute(state).await,
		"wr" => commands::wr::execute(state).await,
		unknown_command => {
			log::warn!("encountered unknown slash command: {}", unknown_command);
			return Ok(());
		},
	}
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

	pub async fn reply(&self, content: InteractionResponseData<'_>) -> anyhow::Result<()> {
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

	fn get(&self, name: &'h str) -> Option<json::Value> {
		if let Some(value) = self.opts.get(name) {
			return Some(value.to_owned());
		}
		return None;
	}

	pub fn get_string(&self, name: &'h str) -> Option<String> {
		if let Some(json::Value::String(string)) = self.get(name) {
			return Some(string);
		}
		return None;
	}

	pub fn get_int(&self, name: &'h str) -> Option<i64> {
		if let Some(json::Value::Number(number)) = self.get(name) {
			return number.as_i64();
		}
		return None;
	}

	pub fn get_float(&self, name: &'h str) -> Option<f64> {
		if let Some(json::Value::Number(number)) = self.get(name) {
			return number.as_f64();
		}
		return None;
	}

	#[allow(dead_code)] // at some point I will use this Copium
	pub fn get_bool(&self, name: &'h str) -> Option<bool> {
		if let Some(json::Value::Bool(boolean)) = self.get(name) {
			return Some(boolean);
		}
		return None;
	}

	pub fn get_user(&self, name: &'h str) -> Option<u64> {
		if let Some(json::Value::String(string)) = self.get(name) {
			if let Ok(user_id) = string.parse::<u64>() {
				return Some(user_id);
			}
		}
		return None;
	}

	pub fn thumbnail(&self, map_name: &String) -> String {
		return format!(
			"https://raw.githubusercontent.com/KZGlobalTeam/map-images/master/images/{}.jpg",
			map_name
		);
	}
}

#[derive(Debug, Clone)]
pub(crate) enum InteractionResponseData<'a> {
	Message(&'a str),
	Embed(CreateEmbed),
}
