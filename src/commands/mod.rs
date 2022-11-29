pub(crate) mod apistatus;
pub(crate) mod bpb;
pub(crate) mod bwr;
pub(crate) mod db;
pub(crate) mod invite;
pub(crate) mod map;
pub(crate) mod mode;
pub(crate) mod nocrouch;
pub(crate) mod pb;
pub(crate) mod ping;
pub(crate) mod profile;
pub(crate) mod random;
pub(crate) mod recent;
pub(crate) mod setsteam;
pub(crate) mod unfinished;
pub(crate) mod wr;

// use {
// 	crate::events::slash_commands::{
// 		InteractionData,
// 		InteractionResponseData::{Message, Embed},
// 	},
// 	gokz_rs::{prelude::*, global_api::*},
// 	serenity::{
// 		builder::{CreateApplicationCommand, CreateEmbed},
// 		model::prelude::command::CommandOptionType,
// 	},
// };
//
// pub(crate) fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
// 	return cmd.name("").description("");
// }
//
// pub(crate) async fn execute(data: InteractionData<'_>) -> anyhow::Result<()> {
// 	return data.reply(Message("hi mom")).await;
// }
