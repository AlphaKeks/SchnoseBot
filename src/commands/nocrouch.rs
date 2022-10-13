use serenity::{
	builder::CreateApplicationCommand,
	json::Value,
	model::prelude::{
		command::CommandOptionType, interaction::application_command::CommandDataOption,
	},
};

use crate::SchnoseCommand;

pub fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	cmd.name("nocrouch")
		.description("Approximate potential distance of a nocrouch jump.")
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

pub fn run(opts: &[CommandDataOption]) -> SchnoseCommand {
	let mut distance = None;
	let mut max = None;

	for opt in opts {
		match opt.name.as_str() {
			"distance" => match &opt.value {
				Some(val) => match val.to_owned() {
					Value::Number(str) => distance = Some(str.as_f64().unwrap_or(0.0)),
					_ => return SchnoseCommand::Message(String::from("Failed to get input.")),
				},
				None => unreachable!("Failed to access required command option"),
			},

			"max" => match &opt.value {
				Some(val) => match val.to_owned() {
					Value::Number(str) => max = Some(str.as_f64().unwrap_or(0.0)),
					_ => return SchnoseCommand::Message(String::from("Failed to get input.")),
				},
				None => unreachable!("Failed to access required command option"),
			},

			_ => (),
		}
	}

	if let (Some(distance), Some(max)) = (distance, max) {
		let approx = distance + (max / 128.0) * 4.0;

		return SchnoseCommand::Message(format!("Approximated distance: `{0:.4}`", approx));
	}

	SchnoseCommand::Message(String::from("Failed to calculate distance."))
}
