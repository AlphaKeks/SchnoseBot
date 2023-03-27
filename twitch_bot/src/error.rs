use std::fmt::Display;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub enum Error {
	Unknown,
	Custom(String),
	NotACommand,
	UnknownCommand(String),
	MissingArgs { missing: String },
	IncorrectArgs { expected: String },
	GOKZ { message: String },
}

impl std::error::Error for Error {}

impl Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Unknown => f.write_str("Unknown error occurred."),
			Self::Custom(message) => f.write_str(message),
			Self::NotACommand => f.write_str(""),
			Self::UnknownCommand(cmd) => f.write_fmt(format_args!("Unknown command `{cmd}`")),
			Self::MissingArgs { missing } => {
				f.write_fmt(format_args!("Missing arguments: {missing}"))
			}
			Self::IncorrectArgs { expected } => {
				f.write_fmt(format_args!("Incorrect arguments. Expected {expected}."))
			}
			Self::GOKZ { message } => f.write_str(message),
		}
	}
}

impl From<gokz_rs::Error> for Error {
	fn from(value: gokz_rs::Error) -> Self {
		Self::GOKZ { message: value.to_string() }
	}
}

impl From<color_eyre::Report> for Error {
	fn from(value: color_eyre::Report) -> Self {
		Self::Custom(value.to_string())
	}
}

impl From<std::convert::Infallible> for Error {
	fn from(_: std::convert::Infallible) -> Self {
		Self::Unknown
	}
}
