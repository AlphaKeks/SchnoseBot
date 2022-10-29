use serenity::{builder::CreateApplicationCommand, model::prelude::command::CommandOptionType};

use crate::event_handler::interaction_create::{CommandOptions, SchnoseResponseData};

pub fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	cmd.name("nocrouch")
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
		})
}

pub fn run<'a>(opts: CommandOptions<'a>) -> SchnoseResponseData {
	let distance = match opts.get_float("distance") {
		Some(num) => num,
		None => unreachable!("option is required"),
	};
	let max = match opts.get_float("max") {
		Some(num) => num,
		None => unreachable!("option is required"),
	};

	let result = distance + (max / 128.0) * 4.0;

	return SchnoseResponseData::Message(format!("Approximated distance: `{0:.4}`", result));
}
