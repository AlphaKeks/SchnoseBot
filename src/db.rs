use {
	crate::schnose::SchnoseErr,
	bson::doc,
	gokz_rs::prelude::Mode,
	serde::{Serialize, Deserialize},
	serenity::model::user::User,
	mongodb::Collection,
};

/// Database schema for user entries
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub(crate) struct UserSchema {
	pub name: String,
	pub discordID: String,
	pub steamID: Option<String>,
	pub mode: Option<String>,
}

/// Utility function to get a user's mode from the database
pub(crate) async fn retrieve_mode(
	user: &User,
	collection: &Collection<UserSchema>,
) -> Result<Mode, SchnoseErr> {
	match collection.find_one(doc! { "discordID": user.id.to_string() }, None).await {
		Ok(document) => {
			if let Some(entry) = document {
				if let Some(mode) = entry.mode {
					// TODO: migrate to a proper database
					if mode.as_str() != "none" {
						let mode =
							mode.parse::<Mode>().expect("This must be valid at this point. `mode_name` can only be valid or \"none\". The latter is already impossible because of the if-statement above.");
						return Ok(mode);
					}
				}
			}
			return Err(SchnoseErr::MissingMode);
		},
		Err(why) => {
			log::error!("[{}]: {} => {:?}", file!(), line!(), why);
			return Err(SchnoseErr::DBAccess);
		},
	}
}
