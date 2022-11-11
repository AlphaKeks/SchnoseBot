use {
	std::{collections::HashMap, env},
	anyhow::Result,
	crate::{commands, Schnose},
	mongodb::Collection,
	serenity::{
		json, http::Http, prelude::Context, builder::CreateEmbed,
		model::application::interaction::application_command::ApplicationCommandInteraction,
	},
};

pub async fn handle(
	_client: &Schnose,
	ctx: Context,
	interaction: ApplicationCommandInteraction,
) -> Result<()> {
	let ctx = InteractionData::new(&interaction, &ctx.http).await?;
	match interaction.data.name.as_str() {
		"ping" => commands::ping::execute(ctx).await,
		unkown_command => unimplemented!("Command `{}` not found.", unkown_command),
	}
}

#[derive(Debug, Clone)]
pub enum InteractionResponseData<'a> {
	Message(&'a str),
	Embed(CreateEmbed),
}

#[derive(Debug, Clone)]
pub struct InteractionData<'a> {
	root: &'a ApplicationCommandInteraction,
	http: &'a Http,
	deferred: bool,
	pub opts: HashMap<String, json::Value>,
	pub db: Collection<crate::db::UserSchema>,
}

impl<'a> InteractionData<'a> {
	async fn new(
		root: &'a ApplicationCommandInteraction,
		http: &'a Http,
	) -> Result<InteractionData<'a>> {
		let mut opts: HashMap<String, json::Value> = HashMap::new();
		for opt in &root.data.options {
			if let Some(value) = opt.value.to_owned() {
				opts.insert(opt.name.clone(), value);
			}
		}

		let mongo_url = env::var("MONGO_URL")?;
		let mongo_options = mongodb::options::ClientOptions::parse(mongo_url).await?;
		let mongo_client = mongodb::Client::with_options(mongo_options)?;

		let db: Collection<crate::db::UserSchema> =
			mongo_client.database("gokz").collection("users");

		return Ok(Self { root, http, deferred: false, opts, db });
	}

	// some commands need to load a bit longer, so we can tell discord to remember an interaction
	pub async fn defer(&mut self) -> Result<()> {
		self.root.defer(self.http).await?;
		self.deferred = true;
		return Ok(());
	}

	pub async fn reply(&self, content: InteractionResponseData<'_>) -> Result<()> {
		if self.deferred {
			self.root
				.edit_original_interaction_response(self.http, |response| match content {
					InteractionResponseData::Message(msg) => response.content(msg),
					InteractionResponseData::Embed(embed) => response.set_embed(embed),
				})
				.await?;
		} else {
			self.root
				.create_interaction_response(self.http, |response| {
					response.interaction_response_data(|response| match content {
						InteractionResponseData::Message(msg) => response.content(msg),
						InteractionResponseData::Embed(embed) => response.set_embed(embed),
					})
				})
				.await?
		}

		return Ok(());
	}

	fn get(&self, name: &'a str) -> Option<json::Value> {
		if let Some(value) = self.opts.get(name) {
			return Some(value.to_owned());
		}
		return None;
	}

	pub fn get_string(&self, name: &'a str) -> Option<String> {
		if let Some(json::Value::String(string)) = self.get(name) {
			return Some(string);
		}
		return None;
	}

	pub fn get_int(&self, name: &'a str) -> Option<i64> {
		if let Some(json::Value::Number(number)) = self.get(name) {
			return number.as_i64();
		}
		return None;
	}

	pub fn get_float(&self, name: &'a str) -> Option<f64> {
		if let Some(json::Value::Number(number)) = self.get(name) {
			return number.as_f64();
		}
		return None;
	}

	pub fn get_bool(&self, name: &'a str) -> Option<bool> {
		if let Some(json::Value::Bool(boolean)) = self.get(name) {
			return Some(boolean);
		}
		return None;
	}
}
