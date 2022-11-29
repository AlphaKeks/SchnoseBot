use {
	crate::events::slash_commands::{InteractionData, InteractionResponseData::Message},
	serenity::{builder::CreateApplicationCommand, model::prelude::command::CommandOptionType},
};

pub(crate) fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	return cmd
		.name("nocrouch")
		.description("Calculates an approximation for the potential distance of an uncrouched LJ.")
		.create_option(|opt| {
			opt.kind(CommandOptionType::Number)
				.name("distance")
				.description("The distance of your jump.")
				.required(true)
		})
		.create_option(|opt| {
			opt.kind(CommandOptionType::Number)
				.name("max")
				.description("The max speed of your jump.")
				.required(true)
		});
}

pub(crate) async fn execute(data: InteractionData<'_>) -> anyhow::Result<()> {
	let distance = data.get_float("distance").expect("This option is marked as `required`.");

	let max = data.get_float("max").expect("This option is marked as `required`.");

	let result = distance + (max / 128f64) * 4f64;

	return data.reply(Message(&format!("Approximated distance: `{0:.4}`", result))).await;
}
