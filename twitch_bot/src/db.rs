use {
	crate::{Error, Result},
	color_eyre::Result as Eyre,
	gokz_rs::{Mode, SteamID, Tier},
	serde::Deserialize,
	sqlx::{FromRow, MySql, Pool, QueryBuilder},
	tracing::warn,
};

#[derive(Debug, FromRow)]
pub struct ConfigRow {
	pub id: u32,
	pub client_id: String,
	pub client_secret: String,
	pub access_token: String,
	pub refresh_token: String,
}

#[derive(Debug, FromRow)]
pub struct ChannelRow {
	pub channel_name: String,
}

#[derive(Debug, Clone)]
pub struct Config {
	pub client_id: String,
	pub client_secret: String,
	pub access_token: String,
	pub refresh_token: String,
	pub channel_names: Vec<String>,
}

pub async fn get_config(conn_pool: &Pool<MySql>, prod: bool) -> Eyre<Config, sqlx::Error> {
	let cfg_id = if prod { 1 } else { 2 };
	dbg!(cfg_id);
	let config: ConfigRow = sqlx::query_as(&format!("SELECT * FROM configs WHERE id = {cfg_id}"))
		.fetch_one(conn_pool)
		.await?;

	let channels: Vec<ChannelRow> = sqlx::query_as("SELECT * FROM twitch_bot_channels")
		.fetch_all(conn_pool)
		.await?;

	Ok(Config {
		client_id: config.client_id,
		client_secret: config.client_secret,
		access_token: config.access_token,
		refresh_token: config.refresh_token,
		channel_names: channels
			.into_iter()
			.map(|row| row.channel_name)
			.collect(),
	})
}

pub async fn update_tokens(
	mut config: Config,
	client: &gokz_rs::Client,
	conn_pool: &Pool<MySql>,
) -> Eyre<Config> {
	let response = client
		.get("https://id.twitch.tv/oauth2/validate")
		.header("Authorization", format!("OAuth {}", config.access_token))
		.send()
		.await?;

	match response.status().as_u16() {
		200 => return Ok(config),
		code => {
			warn!("[{code}] Current `access_token` is not valid anymore. Refreshing...");

			let new_credentials = client
				.post("https://id.twitch.tv/oauth2/token")
				.header("Content-Type", "application/x-www-form-urlencoded")
				.query(&[
					("client_id", config.client_id.as_str()),
					("client_secret", config.client_secret.as_str()),
					("grant_type", "refresh_token"),
					("refresh_token", config.refresh_token.as_str()),
				])
				.send()
				.await?
				.json::<TwitchResponse>()
				.await?;

			let mut query = QueryBuilder::<MySql>::new("UPDATE configs SET access_token = ");

			query
				.push_bind(&new_credentials.access_token)
				.push(" , refresh_token = ")
				.push_bind(&new_credentials.refresh_token)
				.push(" WHERE client_id = ")
				.push_bind(&config.client_id);

			query.build().execute(conn_pool).await?;

			config.access_token = new_credentials.access_token;
			config.refresh_token = new_credentials.refresh_token;
		}
	}

	Ok(config)
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct TwitchResponse {
	access_token: String,
	expires_in: i32,
	refresh_token: String,
	scope: Vec<String>,
	token_type: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct StreamerInfoRow {
	pub api_key: String,
	pub channel_id: u32,
	pub channel_name: String,
	pub player_name: String,
	pub steam_id: String,
	pub mode: Option<String>,
	pub map_name: Option<String>,
	pub map_tier: Option<u8>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StreamerInfo {
	pub api_key: String,
	pub channel_id: u32,
	pub channel_name: String,
	pub player_name: String,
	pub steam_id: SteamID,
	pub mode: Option<Mode>,
	pub map: Option<MapInfo>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MapInfo {
	pub name: String,
	pub tier: Tier,
}

impl TryFrom<StreamerInfoRow> for StreamerInfo {
	type Error = Error;

	fn try_from(value: StreamerInfoRow) -> Result<Self> {
		Ok(Self {
			api_key: value.api_key,
			channel_id: value.channel_id,
			channel_name: value.channel_name,
			player_name: value.player_name,
			steam_id: value.steam_id.parse()?,
			mode: match value.mode {
				Some(mode) => Some(mode.parse()?),
				None => None,
			},
			map: match (value.map_name, value.map_tier) {
				(Some(name), Some(tier)) => Some(MapInfo { name, tier: tier.try_into()? }),
				_ => None,
			},
		})
	}
}
