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
pub mod profile;
pub mod random;
pub mod recent;
pub mod setsteam;
pub mod unfinished;
pub mod wr;

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
// 	return ctx.reply(Message("pong!")).await;
// }
