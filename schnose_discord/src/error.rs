use log::warn;

#[derive(Debug, Clone)]
pub enum SchnoseError {
	Custom(String),
	Serenity(String),
	GOKZ(String),
	InvalidMapName(String),
	InvalidMention(String),
	Parsing(String),
	DatabaseAccess,
	NoDatabaseEntries,
	DatabaseUpdate,
}

impl std::fmt::Display for SchnoseError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let msg = match self {
			SchnoseError::Custom(msg) => msg,
			SchnoseError::Serenity(msg) => msg,
			SchnoseError::GOKZ(msg) => msg,
			SchnoseError::InvalidMapName(map_name) => {
				return write!(f, "`{}` is not a valid map name.", map_name);
			},
			SchnoseError::InvalidMention(mention) => {
				return write!(f, "`{}` is not a valid mention.", mention);
			},
			SchnoseError::Parsing(thing) => {
				return write!(f, "Failed to parse {thing}.");
			},
			SchnoseError::DatabaseAccess => "Failed to access database.",
			SchnoseError::NoDatabaseEntries => "No database entries found.",
			SchnoseError::DatabaseUpdate => "Failed to update database.",
		};
		write!(f, "{}", msg)
	}
}

impl From<serenity::Error> for SchnoseError {
	fn from(value: serenity::Error) -> Self {
		Self::Serenity(value.to_string())
	}
}

impl From<gokz_rs::prelude::Error> for SchnoseError {
	fn from(value: gokz_rs::prelude::Error) -> Self {
		Self::GOKZ(value.msg)
	}
}

impl From<String> for SchnoseError {
	fn from(value: String) -> Self {
		Self::Custom(value)
	}
}

impl From<sqlx::Error> for SchnoseError {
	fn from(value: sqlx::Error) -> Self {
		warn!("DB ERROR `{}`", value);
		match value {
			sqlx::Error::Database(why) => {
				warn!("{}", why);
				Self::DatabaseAccess
			},
			sqlx::Error::RowNotFound => Self::NoDatabaseEntries,
			_ => Self::DatabaseAccess,
		}
	}
}
