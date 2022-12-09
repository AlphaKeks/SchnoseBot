/// MongoDB Database schema for user information
#[allow(non_snake_case)]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct UserSchema {
	pub name: String,
	pub discordID: String,
	pub steamID: Option<String>,
	pub mode: Option<String>
}
