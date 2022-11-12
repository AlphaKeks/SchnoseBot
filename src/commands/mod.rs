pub mod apistatus;
pub mod bpb;
pub mod bwr;
pub mod db;
pub mod invite;
pub mod map;
pub mod mode;
pub mod nocrouch;
pub mod pb;
pub mod ping;

// # Command Template
//
// use {
// 	crate::{
// 		events::slash_command::{
// 			InteractionData,
// 			InteractionResponseData::{Message, Embed},
// 		},
// 		util::*,
// 	},
// 	anyhow::Result,
// 	gokz_rs::{prelude::*, global_api::*},
// 	serenity::{
// 		builder::{CreateApplicationCommand, CreateEmbed},
// 		model::prelude::command::CommandOptionType,
// 	},
// };
//
// pub fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
// 	return cmd.name("ping").description("pong!");
// }
//
// pub async fn execute(ctx: InteractionData<'_>) -> Result<()> {
// 	ctx.reply(Message("pong!")).await?;
// 	return Ok(());
// }
