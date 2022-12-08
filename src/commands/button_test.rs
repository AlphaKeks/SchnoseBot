use serenity::builder::CreateEmbed;

use crate::events::slash_commands::InteractionState;

use {
	crate::{events::slash_commands::InteractionResponseData::*, schnose::InteractionResult},
	serenity::builder::CreateApplicationCommand,
};

pub(crate) fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	return cmd.name("testmedaddy").description("beep");
}

pub(crate) async fn execute(state: &mut InteractionState<'_>) -> InteractionResult {
	state.defer().await?;
	let mut embed = CreateEmbed::default();
	let embeds = (0..10)
		.map(|i| embed.title(format!("page {}", i)).to_owned())
		.collect::<Vec<CreateEmbed>>();
	return Ok(Pagination(embeds));
}
