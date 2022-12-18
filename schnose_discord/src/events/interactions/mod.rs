#![allow(dead_code)]
pub(crate) mod slash_command;

use {
	std::collections::HashMap,
	crate::{
		prelude::{SchnoseError, InteractionResult, InteractionResponseData},
		database::schemas::UserSchema,
	},
	log::{trace, error},
	mongodb::Collection,
	serenity::{
		http::Http,
		model::{
			user::User, prelude::interaction::application_command::ApplicationCommandInteraction,
		},
		json,
	},
};

/// Data which lives as long as it's associated Interaction
/// -> will be passed to most commands
#[derive(Debug, Clone)]
pub(crate) struct InteractionState<'a> {
	http: &'a Http,
	// current Discord Interaction
	interaction: &'a ApplicationCommandInteraction,
	// Interaction options (user input)
	opts: HashMap<String, json::Value>,
	// whether the current Interaction has been deferred or not
	deferred: bool,
	// whether the current Interaction has been ephemeralized or not
	ephemeral: bool,
	// the User who triggered this Interaction
	pub user: &'a User,
	// reference to the MongoDB Database collection stored in `GlobalState`
	pub db: &'a Collection<UserSchema>,
	// reference to the reqwest Client stored in `GlobalState`
	pub req_client: &'a gokz_rs::Client,
	// #7480c2
	pub colour: (u8, u8, u8),
	// Icon URL for embed footers
	pub icon: &'a String,
}

impl<'a> InteractionState<'a> {
	pub fn new(
		http: &'a Http,
		interaction: &'a ApplicationCommandInteraction,
		database_collection: &'a Collection<UserSchema>,
		req_client: &'a gokz_rs::Client,
		colour: (u8, u8, u8),
		icon: &'a String,
	) -> InteractionState<'a> {
		let mut opts: HashMap<String, json::Value> = HashMap::new();

		// filter out relevant data from Interaction data and store it for later access
		for option in &interaction.data.options {
			if let Some(value) = option.value.to_owned() {
				opts.insert(option.name.clone(), value);
			}
		}

		return InteractionState {
			http,
			interaction,
			opts,
			deferred: false,
			ephemeral: false,
			user: &interaction.user,
			db: database_collection,
			req_client,
			colour,
			icon,
		};
	}

	/// Wrapper function to defer the current interaction's response
	pub async fn defer(&mut self) -> Result<(), SchnoseError> {
		if let Err(why) = self.interaction.defer(self.http).await {
			error!("Failed to defer interaction. {:?}", why);
			return Err(SchnoseError::Defer);
		}

		self.deferred = true;
		trace!("Deferred Interaction `{}`.", &self.interaction.data.name);

		return Ok(());
	}

	/// Wrapper function to ephemeralize the current interaction's response
	/// Note: this only works for non-deferred messages.
	pub fn ephemeralize(&mut self) {
		self.ephemeral = true;
	}

	/// Wrapper function around `self.opts` to easily retrieve user input
	pub fn get<T>(&self, name: &str) -> Option<T>
	where
		T: serde::de::DeserializeOwned,
	{
		return match self.opts.get(name) {
			Some(value) => serde_json::from_value(value.to_owned()).ok(),
			None => None,
		};
	}

	/// Convenience function for linking to a map's KZGO page
	pub fn map_link(&self, map_name: &str) -> String {
		return format!("https://kzgo.eu/maps/{}", map_name);
	}

	/// Convenience function for displaying a map's thumbnail
	pub fn map_thumbnail(&self, map_name: &str) -> String {
		return format!(
			"https://raw.githubusercontent.com/KZGlobalTeam/map-images/master/images/{}.jpg",
			map_name
		);
	}

	async fn reply(&self, content: InteractionResult) -> anyhow::Result<()> {
		let content = match content {
			Ok(reply) => {
				trace!("Successful Interaction Response: {:?}", &reply);
				reply
			},
			Err(why) => {
				error!("Failed Interaction Response: {:?}", &why);
				why.into()
			},
		};

		// Interaction has been deferred => edit original message
		if self.deferred {
			match self
				.interaction
				.edit_original_interaction_response(self.http, |response| match content {
					InteractionResponseData::Message(message) => {
						response.content(message);

						response
					},
					InteractionResponseData::Embed(embed) => {
						response.set_embed(embed);

						response
					},
				})
				.await
			{
				Ok(_) => {
					trace!("responded to Interaction `{}`.", &self.interaction.data.name);
				},
				Err(why) => {
					error!(
						"failed to respond to Interaction `{}`: {:?}",
						&self.interaction.data.name, why
					);
				},
			}
		} else {
			match self
				.interaction
				.create_interaction_response(self.http, |response| {
					response.interaction_response_data(|response| {
						if self.ephemeral {
							response.ephemeral(true);
						}

						match content {
							InteractionResponseData::Message(message) => {
								response.content(message);

								response
							},
							InteractionResponseData::Embed(embed) => {
								response.set_embed(embed);

								response
							},
						}
					})
				})
				.await
			{
				Ok(_) => {
					trace!("responded to Interaction `{}`.", &self.interaction.data.name);
				},
				Err(why) => {
					error!(
						"failed to respond to Interaction `{}`: {:?}",
						&self.interaction.data.name, why
					);
				},
			}
		}

		return Ok(());
	}
}
