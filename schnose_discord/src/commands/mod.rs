pub(crate) mod apistatus;
pub(crate) mod bmaptop;
pub(crate) mod bpb;
pub(crate) mod bwr;
pub(crate) mod db;
pub(crate) mod invite;
pub(crate) mod map;
pub(crate) mod maptop;
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

// Example command structure
// use {crate::InteractionResult, serenity::builder::CreateApplicationCommand};
//
// pub(crate) fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
// 	return cmd.name("ping").description("pong!");
// }
//
// pub(crate) async fn execute() -> InteractionResult {
// 	return Ok("pong!".into());
// }
