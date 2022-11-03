#![allow(dead_code)] // TODO: remove this later
use std::{collections::HashMap, env};

use mongodb::{
	options::{ClientOptions, ResolverConfig},
	Client,
};
use serenity::{
	builder::CreateEmbed,
	json,
	model::{
		application::interaction::Interaction,
		prelude::interaction::application_command::ApplicationCommandInteraction,
	},
	prelude::Context,
};

use crate::{commands, util::UserSchema};

// TODO: make this more flexible
pub enum SchnoseResponseData {
	Message(String),
	Embed(CreateEmbed),
	// maybe something like this?
	// Pagination(Vec<CreateEmbed>),
}

pub struct Metadata {
	pub ctx: Context,
	pub cmd: ApplicationCommandInteraction,
	pub opts: CommandOptions,
}

impl Metadata {
	pub async fn reply(&self, response_data: SchnoseResponseData) {
		let result = match response_data {
			// SchnoseResponseData::Pagination(embed_list) => unimplemented!(),
			_ => {
				self.cmd
					.edit_original_interaction_response(&self.ctx.http, |response| {
						match response_data {
							SchnoseResponseData::Message(msg) => response.content(msg),
							SchnoseResponseData::Embed(embed) => response.set_embed(embed),
							// _ => unreachable!(),
						}
					})
					.await
			},
		};

		if let Err(why) = result {
			log::error!(
				"[{}]: {} => {}\n{:#?}",
				file!(),
				line!(),
				"Failed to respond to pagination.",
				why
			);
		}
	}
}

#[derive(Debug, Clone)]
pub struct CommandOptions(HashMap<String, json::Value>);

impl<'a> CommandOptions {
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

pub async fn handle(root: &crate::Schnose, ctx: Context, interaction: Interaction) {
	let mongodb_client = match env::var("MONGODB") {
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
			if let Err(why) = &cmd.defer(&ctx.http).await {
				return log::error!(
					"[{}]: {} => {}\n{:#?}",
					file!(),
					line!(),
					"Failed to defer command.",
					why
				);
			}

			log::info!("Received interaction: {}", &cmd.data.name);

			let mut metadata = Metadata { ctx, cmd, opts: CommandOptions(HashMap::new()) };
			for opt in metadata.cmd.data.options.clone() {
				if let Some(value) = opt.value.clone() {
					metadata.opts.0.insert(opt.name, value);
				}
			}

			let collection = mongodb_client.database("gokz").collection::<UserSchema>("users");

			match metadata.cmd.data.name.as_str() {
				"ping" => commands::ping::run(metadata).await,
				"invite" => commands::invite::run(metadata).await,
				"setsteam" => commands::setsteam::run(metadata, &collection).await,
				"mode" => commands::mode::run(metadata, &collection).await,
				"db" => commands::db::run(metadata, &collection).await,
				"nocrouch" => commands::nocrouch::run(metadata).await,
				"apistatus" => commands::apistatus::run(metadata).await,
				"bpb" => commands::bpb::run(metadata, &collection, root).await,
				"pb" => commands::pb::run(metadata, &collection, root).await,
				"bwr" => commands::bwr::run(metadata, &collection, root).await,
				"wr" => commands::wr::run(metadata, &collection, root).await,
				"recent" => commands::recent::run(metadata, &collection, root).await,
				"unfinished" => commands::unfinished::run(metadata, &collection, root).await,
				"random" => commands::random::run(metadata).await,
				"map" => commands::map::run(metadata).await,
				"profile" => commands::profile::run(metadata, &collection, root).await,
				unknown_command => unimplemented!("Command `{}` not found.", unknown_command),
			}

			// prepare response
			/*
			let data = match cmd.data.name.as_str() {
				"ping" => commands::ping::run(),
				"invite" => commands::invite::run(),
				"setsteam" => commands::setsteam::run(cmd_opts, &collection, &cmd.user).await,
				"mode" => commands::mode::run(cmd_opts, &collection, &cmd.user).await,
				"db" => commands::db::run(&collection, &cmd.user).await,
				"nocrouch" => commands::nocrouch::run(cmd_opts),
				"apistatus" => commands::apistatus::run().await,
				"bpb" => commands::bpb::run(cmd_opts, &collection, &cmd.user, root).await,
				"pb" => commands::pb::run(cmd_opts, &collection, &cmd.user, root).await,
				"bwr" => commands::bwr::run(cmd_opts, &collection, &cmd.user, root).await,
				"wr" => commands::wr::run(cmd_opts, &collection, &cmd.user, root).await,
				"recent" => commands::recent::run(cmd_opts, &collection, &cmd.user, root).await,
				"unfinished" => {
					commands::unfinished::run(cmd_opts, &collection, &cmd.user, root).await
				},
				"random" => commands::random::run(cmd_opts).await,
				"map" => commands::map::run(cmd_opts).await,
				"profile" => commands::profile::run(cmd_opts, &collection, &cmd.user, root).await,
				unknown_command => unimplemented!("Command `{}` not found.", unknown_command),
			};
			*/

			/*
			if let Err(why) = cmd
				// respond to user
				.edit_original_interaction_response(&ctx.http, |response| match data {
					SchnoseResponseData::Message(msg) => response.content(msg),
					SchnoseResponseData::Embed(embed) => response.set_embed(embed),
					SchnoseResponseData::Pagination(_embed_list) => todo!(),
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
			*/
		},
		_ => unimplemented!(),
	}
}
