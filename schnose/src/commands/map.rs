use serenity::builder::CreateApplicationCommand;

pub fn data(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	cmd.name("map").description("map!")
}

pub fn execute() -> String {
	String::from("map!")
}
