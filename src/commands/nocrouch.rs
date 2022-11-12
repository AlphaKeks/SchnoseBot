use {
	crate::events::slash_command::{InteractionData, InteractionResponseData::Message},
	anyhow::Result,
	serenity::{builder::CreateApplicationCommand, model::prelude::command::CommandOptionType},
};

pub fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	return cmd
		.name("nocrouch")
		.description("Calculates an approximation for the potential distance of an uncrouched LJ.")
		.create_option(|opt| {
			opt.kind(CommandOptionType::Number)
				.name("distance")
				.description("The distance of your jump")
				.required(true)
		})
		.create_option(|opt| {
			opt.kind(CommandOptionType::Number)
				.name("max")
				.description("The max speed of your jump")
				.required(true)
		});
}

pub async fn execute(ctx: InteractionData<'_>) -> Result<()> {
	let distance = ctx.get_float("distance").expect("This option is marked as `required`.");
	let max = ctx.get_float("max").expect("This option is marked as `required`.");
	let result = distance + (max / 128.0) * 4.0;

	return ctx.reply(Message(&format!("Approximated distance: `{0:.4}`", result))).await;
}
