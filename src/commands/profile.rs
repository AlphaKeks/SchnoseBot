use {
	crate::{
		events::slash_commands::{InteractionState, InteractionResponseData::*},
		schnose::{InteractionResult, Target},
		util::*,
		db::retrieve_mode,
	},
	bson::doc,
	gokz_rs::{prelude::*, global_api::*, custom::get_profile, kzgo},
	num_format::{ToFormattedString, Locale},
	serenity::{
		builder::{CreateApplicationCommand, CreateEmbed},
		model::prelude::command::CommandOptionType,
	},
};

pub(crate) fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	return cmd
		.name("profile")
		.description("Check a player's stats.")
		.create_option(|opt| {
			opt.kind(CommandOptionType::String)
				.name("mode")
				.description("Specify a mode.")
				.add_string_choice("KZT", "kz_timer")
				.add_string_choice("SKZ", "kz_simple")
				.add_string_choice("VNL", "kz_vanilla")
				.required(false)
		})
		.create_option(|opt| {
			opt.kind(CommandOptionType::String)
				.name("player")
				.description("Specify a player.")
				.required(false)
		});
}

pub(crate) async fn execute(state: &mut InteractionState<'_>) -> InteractionResult {
	// Defer current interaction since this could take a while
	state.defer().await?;

	let target = Target::from(state.get::<String>("player"));

	let player = target.to_player(state.user, state.db).await?;

	let mode = match state.get::<String>("mode") {
		Some(mode_name) => Mode::from_str(&mode_name)
			.expect("The possible values for this are hard-coded and should never be invalid."),
		None => retrieve_mode(state.user, state.db).await?,
	};

	let player = get_player(&player, &state.req_client).await?;

	let steam_id = SteamID::new(&player.steam_id)?;

	let player_profile =
		get_profile(&PlayerIdentifier::SteamID(steam_id), &mode, &state.req_client).await?;

	let avatar = get_steam_avatar(&player_profile.steam_id64, &state.req_client).await;

	let fav_mode = {
		let mut mode = String::new();
		if let Ok(Some(entry)) =
			state.db.find_one(doc! { "steamID": &player_profile.steam_id }, None).await
		{
			if let Some(db_mode) = entry.mode {
				if db_mode != "none" {
					mode = Mode::from_str(&db_mode)
						.expect("This must be valid at this point. `mode_name` can only be valid or \"none\". The latter is already impossible because of the if-statement above.")
						.to_fancy();
				}
			}
		}

		mode
	};

	let mut bars = [[""; 7]; 2].map(|a| a.map(|s| s.to_owned()));

	for i in 0..7 {
		{
			let amount = (&player_profile.completion_percentage[i].0 / 10.0) as u32;

			for _ in 0..amount {
				bars[0][i].push_str("█");
			}

			for _ in 0..(10 - amount) {
				bars[0][i].push_str("░");
			}
		}

		{
			let amount = (&player_profile.completion_percentage[i].1 / 10.0) as u32;

			for _ in 0..amount {
				bars[1][i].push_str("█");
			}

			for _ in 0..(10 - amount) {
				bars[1][i].push_str("░");
			}
		}
	}

	// how many maps _can_ be finished in a certain mode?
	let doable = match kzgo::completion::get_completion_count(&mode, &state.req_client).await {
		Ok(data) => (data.tp.total, data.pro.total),
		Err(why) => {
			log::warn!("[{}]: {} => {:?}", file!(), line!(), why);
			return Err(why.into());
		},
	};

	let embed = CreateEmbed::default()
		.colour(state.colour)
		.title(format!(
			"{} - {} Profile",
			&player_profile.name.unwrap_or(String::from("unknown")),
			&mode.to_fancy()
		))
		.url(format!(
			"https://kzgo.eu/players/{}?{}=",
			match &player_profile.steam_id {
				Some(steam_id) => steam_id,
				None => "unknown",
			},
			&mode.to_fancy().to_lowercase()
		))
		.thumbnail(avatar)
		// this is so incredibly whacky I'm scared to touch it ever again
		.description(format!(
			r#"
🏆 **World Records: {} (TP) | {} (PRO)**
────────────────────────
                      TP                                               PRO
         `{}/{} ({:.2}%)`              `{}/{} ({:.2}%)`
T1     ⌠ {} ⌡          ⌠ {} ⌡
T2   ⌠ {} ⌡          ⌠ {} ⌡
T3   ⌠ {} ⌡          ⌠ {} ⌡
T4  ⌠ {} ⌡          ⌠ {} ⌡
T5   ⌠ {} ⌡          ⌠ {} ⌡
T6  ⌠ {} ⌡          ⌠ {} ⌡
T7   ⌠ {} ⌡          ⌠ {} ⌡

Points: **{}**
────────────────────────
Rank: **{}**
Preferred Mode: {}
			"#,
			&player_profile.records.0,
			&player_profile.records.1,
			&player_profile.completion[7].0,
			&doable.0,
			&player_profile.completion_percentage[7].0,
			&player_profile.completion[7].1,
			&doable.1,
			&player_profile.completion_percentage[7].1,
			&bars[0][0],
			&bars[1][0],
			&bars[0][1],
			&bars[1][1],
			&bars[0][2],
			&bars[1][2],
			&bars[0][3],
			&bars[1][3],
			&bars[0][4],
			&bars[1][4],
			&bars[0][5],
			&bars[1][5],
			&bars[0][6],
			&bars[1][6],
			(&player_profile.points.0 + &player_profile.points.1).to_formatted_string(&Locale::en),
			match &player_profile.rank {
				Some(rank) => rank.to_string(),
				None => String::from("unknown"),
			},
			fav_mode
		))
		.footer(|f| {
			f.text(format!(
				"SteamID: {}",
				&player_profile.steam_id.unwrap_or(String::from("unknown"))
			))
			.icon_url(&state.icon)
		})
		.to_owned();

	return Ok(Embed(embed));
}
