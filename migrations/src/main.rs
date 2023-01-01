use sqlx::mysql::MySqlPoolOptions;

#[tokio::main]
async fn main() {
	dotenv::dotenv().expect("Failed to initialize .env");

	let db_url = std::env::var("DATABASE_URL").expect("Missing `DATABASE_URL`.");

	let pool = MySqlPoolOptions::new()
		.max_connections(10)
		.connect(&db_url)
		.await
		.expect("Failed to connect to the database.");

	let users = include_str!("../users.json");
	let users: Vec<OldSchema> = serde_json::from_str(users).expect("Failed to parse users.");

	let users: Vec<UserSchema> = users
		.into_iter()
		.map(|user| UserSchema {
			name: user.name,
			discord_id: user.discordID,
			steam_id: user.steamID,
			mode: match user.mode {
				Some(mode) if mode == "none" => None,
				mode => mode,
			},
		})
		.collect();

	let mut query = format!(
		r#"INSERT INTO discord_users (name, discord_id, steam_id, mode) VALUES ("{}", {}, {}, {})"#,
		users[0].name,
		users[0].discord_id,
		match &users[0].steam_id {
			Some(steam_id) => format!(r#""{}""#, steam_id),
			None => String::from("NULL"),
		},
		match &users[0].mode {
			Some(mode) => format!(
				r#"{}"#,
				match mode.as_str() {
					"kz_timer" => 200,
					"kz_simple" => 201,
					"kz_vanilla" => 202,
					_ => unreachable!(),
				}
			),
			None => String::from("NULL"),
		}
	);

	for UserSchema { name, discord_id, steam_id, mode } in users.iter().skip(1) {
		query.push_str(&format!(
			r#",("{}", {}, {}, {})"#,
			name,
			discord_id,
			match steam_id {
				Some(steam_id) => format!(r#""{}""#, steam_id),
				None => String::from("NULL"),
			},
			match mode {
				Some(mode) => format!(
					r#"{}"#,
					match mode.as_str() {
						"kz_timer" => 200,
						"kz_simple" => 201,
						"kz_vanilla" => 202,
						_ => unreachable!(),
					}
				),
				None => String::from("NULL"),
			}
		));
	}

	if let Err(why) = sqlx::query(&query).execute(&pool).await {
		panic!("FUCK {}", why);
	}
}

#[allow(non_snake_case)]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct OldSchema {
	name: String,
	discordID: String,
	steamID: Option<String>,
	mode: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct UserSchema {
	name: String,
	discord_id: String,
	steam_id: Option<String>,
	mode: Option<String>,
}
