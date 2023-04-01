use {
	gokz_rs::{MapIdentifier, Mode, PlayerIdentifier},
	std::fmt::Display,
	tracing::error,
};

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
	Database(DatabaseError),
	Twitch,
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
			Self::Database(db_err) => match db_err {
				DatabaseError::StreamerNotFound => {
					f.write_str("Streamer is not in the database. Please supply arguments.")
				}
				DatabaseError::Other => f.write_str("Database error."),
			},
			Self::Twitch => f.write_str("Twitch API error."),
		}
	}
}

impl From<&Error> for Error {
	fn from(value: &Error) -> Self {
		value.to_owned()
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

impl From<sqlx::Error> for Error {
	fn from(value: sqlx::Error) -> Self {
		error!("Database error: {value:#?}");
		Self::Database(DatabaseError::Other)
	}
}

impl From<twitch_irc::validate::Error> for Error {
	fn from(value: twitch_irc::validate::Error) -> Self {
		error!("Twitch error: {value:#?}");
		Self::Twitch
	}
}

#[derive(Debug, Clone, Copy)]
pub enum DatabaseError {
	StreamerNotFound,
	Other,
}

pub trait GenParseError {
	fn incorrect() -> crate::Error;
}

macro_rules! gen_parse_err {
	($t:ty, $incorrect:expr) => {
		impl GenParseError for $t {
			fn incorrect() -> crate::Error {
				$incorrect
			}
		}
	};
}

pub(crate) use gen_parse_err;

gen_parse_err!(Mode, crate::Error::IncorrectArgs { expected: String::from("mode") });
gen_parse_err!(PlayerIdentifier, crate::Error::IncorrectArgs { expected: String::from("player") });
gen_parse_err!(MapIdentifier, crate::Error::IncorrectArgs { expected: String::from("map") });
