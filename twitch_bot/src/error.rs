use std::fmt::Display;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub enum Error {
	UnknownCommand(String),
	NoArgs(NoArgs),
	GOKZ { message: String },
	MapNotGlobal,
}

impl Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::UnknownCommand(cmd) => f.write_fmt(format_args!("Unknown command `{cmd}`")),
			Self::NoArgs(kind) => {
				f.write_fmt(format_args!("You provided incorrect arguments. Expected {kind}"))
			}
			Self::GOKZ { message } => f.write_str(message),
			Self::MapNotGlobal => f.write_str("The provided map is not global."),
		}
	}
}

impl From<gokz_rs::Error> for Error {
	fn from(value: gokz_rs::Error) -> Self {
		Self::GOKZ { message: value.to_string() }
	}
}

#[derive(Debug, Clone, Copy)]
pub enum NoArgs {
	Map,
}

impl From<NoArgs> for Error {
	fn from(value: NoArgs) -> Self {
		Self::NoArgs(value)
	}
}

impl Display for NoArgs {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		// "You provided incorrect arguments. Expected ..."
		match self {
			NoArgs::Map => f.write_str("map name"),
		}
	}
}
