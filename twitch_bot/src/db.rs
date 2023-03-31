use sqlx::{MySql, Pool, QueryBuilder};

use {color_eyre::Result as Eyre, serde::Deserialize, sqlx::FromRow, tracing::warn};

#[derive(Debug, FromRow)]
pub struct ConfigRow {
	pub client_id: String,
	pub client_secret: String,
	pub access_token: String,
	pub refresh_token: String,
}

#[derive(Debug, FromRow)]
pub struct ChannelRow {
	pub channel_name: String,
}

pub struct Config {
	pub client_id: String,
	pub client_secret: String,
	pub access_token: String,
	pub refresh_token: String,
	pub channel_names: Vec<String>,
}

pub async fn get_config(conn_pool: &Pool<MySql>) -> Result<Config, sqlx::Error> {
	let config: ConfigRow = sqlx::query_as("SELECT * FROM configs LIMIT 1")
		.fetch_one(conn_pool)
		.await?;

	let channels: Vec<ChannelRow> = sqlx::query_as("SELECT * FROM channels")
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
