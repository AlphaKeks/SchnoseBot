use serde::{Serialize, Deserialize};

#[allow(dead_code, non_snake_case)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSchema {
	pub name: String,
	pub discordID: String,
	pub steamID: Option<String>,
	pub mode: Option<String>,
}
