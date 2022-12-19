use {
	super::schemas::UserSchema, crate::prelude::SchnoseError, log::error, bson::doc,
	gokz_rs::prelude::*, mongodb::Collection, serenity::model::user::User,
};

pub(crate) async fn fetch_mode(
	user: &User,
	database: &Collection<UserSchema>,
	blame_user: bool,
) -> Result<Mode, SchnoseError> {
	match database.find_one(doc! { "discordID": user.id.to_string() }, None).await {
		Ok(document) => match document {
			Some(entry) => match entry.mode {
				Some(mode_name) if mode_name != "none" => {
					let mode: Mode = mode_name.parse().expect("This must be valid.");
					Ok(mode)
				},
				_ => Err(SchnoseError::MissingMode(blame_user)),
			},
			None => Err(SchnoseError::MissingDBEntry(blame_user)),
		},
		Err(why) => {
			error!("{}", why.to_string());
			Err(SchnoseError::DBAccess)
		},
	}
}
