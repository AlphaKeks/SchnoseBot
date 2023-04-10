use {
	color_eyre::Result,
	gokz_rs::{global_api, MapIdentifier, Mode, SteamID, Tier},
	serde::{Deserialize, Serialize, Serializer},
};

/// Global state of the game. This is stored in an `Arc<Mutex<State>>` to keep in sync with all
/// parts of the application.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct State {
	#[serde(flatten)]
	pub player: Option<Player>,
	pub map: Map,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Player {
	#[serde(rename = "player_name")]
	pub name: String,
	pub steam_id: SteamID,
	#[serde(serialize_with = "serialize_mode_for_js")]
	pub mode: Option<Mode>,
}

fn serialize_mode_for_js<S>(mode: &Option<Mode>, serializer: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	mode.map(|mode| mode.short())
		.serialize(serializer)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Map {
	pub name: String,
	pub tier: Option<Tier>,
}

impl State {
	/// Parse a [`schnose_gsi::Event`] into [`Self`], fetching information from the GlobalAPI. This
	/// should not be called in short intervals, or you will get rate-limited.
	pub async fn new(event: schnose_gsi::Event, client: &gokz_rs::Client) -> Result<Self> {
		let player = event.player.map(|e_player| Player {
			name: e_player.name.clone(),
			steam_id: e_player.steam_id,
			mode: e_player
				.clan
				.and_then(|clan| match clan.contains(' ') {
					// [SKZ Beginner] Player Name
					true => clan
						.split_once(' ')
						.and_then(|(mode, _rank)| {
							mode.replace('[', "")
								.parse::<Mode>()
								.ok()
						}),
					// [SKZ] Player Name
					false => clan
						.replace(['[', ']'], "")
						.parse::<Mode>()
						.ok(),
				}),
		});

		let map = match event.map.map(|map| {
			if map.name.contains('/') {
				map.name
					.rsplit_once('/')
					.map(|(_, map_name)| map_name.to_owned())
					.unwrap()
			} else {
				map.name.clone()
			}
		}) {
			None => Map {
				name: String::from("unknown map"),
				tier: None,
			},
			Some(map_name) if !Self::is_valid_map_name(&map_name) => Map {
				name: String::from("unknown map"),
				tier: None,
			},
			Some(map_name) => global_api::get_map(&MapIdentifier::Name(map_name), client)
				.await
				.map(|map| Map {
					name: map.name,
					tier: Some(map.difficulty),
				})?,
		};

		Ok(Self { player, map })
	}

	fn is_valid_map_name(map_name: &str) -> bool {
		[
			"kz_", "kzpro_", "bkz_", "xc_", "skz_", "vnl_",
		]
		.iter()
		.any(|prefix| map_name.starts_with(prefix))
	}
}
