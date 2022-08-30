use serenity::builder::CreateApplicationCommand;

pub fn data(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	cmd.name("ping").description("pong!")
}

pub fn execute() -> String {
	String::from("pong!")
}
