#![allow(dead_code)] // TODO: remove this later
use std::{collections::HashMap, env};

use mongodb::{
	options::{ClientOptions, ResolverConfig},
	Client,
};
use serenity::{
	builder::CreateEmbed, json, model::application::interaction::Interaction, prelude::Context,
};

use crate::commands;

// TODO: make this more flexible
pub enum SchnoseResponseData {
	Message(String),
	Embed(CreateEmbed),
	// maybe something like this?
	// Pagination(Vec<CreateEmbed>)
}

pub struct CommandOptions<'a>(HashMap<&'a str, json::Value>);

impl<'a> CommandOptions<'a> {
	fn get(&self, name: &'a str) -> Option<json::Value> {
		match self.0.get(name) {
			None => return None,
			Some(value) => return Some(value.to_owned()),
		}
	}

	pub fn get_string(&self, name: &'a str) -> Option<String> {
		match self.get(name) {
			None => None,
			Some(value) => match value {
				json::Value::String(string) => Some(string),
				_ => None,
			},
		}
	}

	pub fn get_int(&self, name: &'a str) -> Option<i64> {
		match self.get(name) {
			None => None,
			Some(value) => match value {
				json::Value::Number(number) => number.as_i64(),
				_ => None,
			},
		}
	}

	pub fn get_float(&self, name: &'a str) -> Option<f64> {
		match self.get(name) {
			None => None,
			Some(value) => match value {
				json::Value::Number(number) => number.as_f64(),
				_ => None,
			},
		}
	}

	pub fn get_bool(&self, name: &'a str) -> Option<bool> {
		match self.get(name) {
			None => None,
			Some(value) => match value {
				json::Value::Bool(bool) => Some(bool),
				_ => None,
			},
		}
	}
}

pub async fn handle(_root: &crate::Schnose, ctx: Context, interaction: Interaction) {
	let _mongodb_client = match env::var("MONGODB") {
		Err(_) => panic!("No `MONGODB` variable found."),
		Ok(token) => {
			match ClientOptions::parse_with_resolver_config(token, ResolverConfig::cloudflare())
				.await
			{
				Err(why) => panic!("Failed to connect to MongoDB {:#?}", why),
				Ok(client_options) => match Client::with_options(client_options) {
					Err(why) => panic!("Failed to create MongoDB client. {:#?}", why),
					Ok(client) => client,
				},
			}
		},
	};

	match interaction {
		Interaction::ApplicationCommand(cmd) => {
			// try to defer command (some take too long because of API requests)
			if let Err(why) = cmd.defer(&ctx.http).await {
				return log::error!(
					"[{}]: {} => {}\n{:#?}",
					file!(),
					line!(),
					"Failed to defer command.",
					why
				);
			}

			log::info!("Received interaction: {}", &cmd.data.name);

			let cmd_data = &cmd.data;
			let mut cmd_opts = CommandOptions(HashMap::new());
			for opt in &cmd_data.options {
				if let Some(value) = opt.value.clone() {
					cmd_opts.0.insert(opt.name.as_str(), value);
				}
			}

			let data = match cmd.data.name.as_str() {
				"ping" => commands::ping::run(),
				"invite" => commands::invite::run(),
				unknown_command => unimplemented!("Command `{}` not found.", unknown_command),
			};

			if let Err(why) = cmd
				.edit_original_interaction_response(&ctx.http, |response| match data {
					SchnoseResponseData::Message(msg) => response.content(msg),
					SchnoseResponseData::Embed(embed) => response.set_embed(embed),
				})
				.await
			{
				log::error!(
					"[{}]: {} => {}\n{:#?}",
					file!(),
					line!(),
					"Failed to respond to interaction.",
					why
				);
			}
		},
		_ => unimplemented!(),
	}
}
