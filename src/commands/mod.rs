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

// Syntax for new command:
// use serenity::builder::CreateApplicationCommand;
//
// use crate::event_handler::interaction_create::SchnoseResponseData;
//
// pub fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
// 	cmd.name("").description("")
// }
//
// pub fn run() -> SchnoseResponseData {
// 	todo!()
// }
