#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct UserSchema {
	pub name: String,
	pub discord_id: u64,
	pub steam_id: Option<String>,
	pub mode: Option<u8>,
}
