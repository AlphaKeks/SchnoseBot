use {
	crate::schnose::SchnoseErr,
	bson::doc,
	gokz_rs::prelude::Mode,
	serde::{Serialize, Deserialize},
	serenity::model::user::User,
	mongodb::Collection,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub(crate) struct UserSchema {
	pub name: String,
	pub discordID: String,
	pub steamID: Option<String>,
	pub mode: Option<String>,
}

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
							Mode::from_str(&mode).expect("This must be valid at this point.");
						return Ok(mode);
					}
				}
			}
			return Err(SchnoseErr::NoModeSpecified);
		},
		Err(why) => {
			log::error!("[{}]: {} => {:?}", file!(), line!(), why);
			return Err(SchnoseErr::FailedDB);
		},
	}
}
