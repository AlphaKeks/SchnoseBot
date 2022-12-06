use {
	crate::{
		events::slash_commands::{
			GlobalState,
			InteractionResponseData::{self, *},
		},
		schnose::Target,
		db::retrieve_mode,
	},
	gokz_rs::{prelude::*, global_api::*},
	serenity::{
		builder::{CreateApplicationCommand, CreateEmbed},
		model::prelude::command::CommandOptionType,
	},
};

pub(crate) fn register(cmd: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	return cmd
		.name("unfinished")
		.description("Check which maps you still need to complete!")
		.create_option(|opt| {
			opt.kind(CommandOptionType::String)
				.name("mode")
				.description("Choose a mode.")
				.add_string_choice("KZT", "kz_timer")
				.add_string_choice("SKZ", "kz_simple")
				.add_string_choice("VNL", "kz_vanilla")
				.required(false)
		})
		.create_option(|opt| {
			opt.kind(CommandOptionType::String)
				.name("runtype")
				.description("TP/PRO")
				.add_string_choice("TP", "true")
				.add_string_choice("PRO", "false")
				.required(false)
		})
		.create_option(|opt| {
			opt.kind(CommandOptionType::Integer)
				.name("tier")
				.description("Filter by tier")
				.add_int_choice("1 (Very Easy)", 1)
				.add_int_choice("2 (Easy)", 2)
				.add_int_choice("3 (Medium)", 3)
				.add_int_choice("4 (Hard)", 4)
				.add_int_choice("5 (Very Hard)", 5)
				.add_int_choice("6 (Extreme)", 6)
				.add_int_choice("7 (Death)", 7)
				.required(false)
		})
		.create_option(|opt| {
			opt.kind(CommandOptionType::String)
				.name("player")
				.description("Specify a player.")
				.required(false)
		});
}

pub(crate) async fn execute(
	state: &mut GlobalState<'_>,
) -> anyhow::Result<InteractionResponseData> {
	// Defer current interaction since this could take a while
	state.defer().await?;

	let target = Target::from(state.get::<String>("player"));

	let player = match target.to_player(state.user, state.db).await {
		Ok(player) => player,
		Err(why) => {
			log::warn!("[{}]: {} => {:?}", file!(), line!(), why);
			return Ok(Message(why.to_string()));
		},
	};

	let mode = match state.get::<String>("mode") {
		Some(mode_name) => Mode::from_str(&mode_name)
			.expect("The possible values for this are hard-coded and should never be invalid."),
		None => match retrieve_mode(state.user, state.db).await {
			Ok(mode) => mode,
			Err(why) => {
				log::warn!("[{}]: {} => {:?}", file!(), line!(), why);
				return Ok(Message(why.to_string()));
			},
		},
	};

	let runtype = match state.get::<String>("runtype") {
		// Discord supports booleans as parameters for slash commands, however you cannot customize
		// the prompt for the user. It will simply be "True" or "False" which is not ideal. That's
		// why we use Strings. It's fine though since these values are hard-coded.		// why we use Strings. It's fine though since these values are hard-coded.		// why we use Strings. It's fine though since these values are hard-coded.
		Some(runtype) => match runtype.as_str() {
			"true" => true,
			"false" => false,
			_ => unreachable!("only `true` and `false` exist as selectable options."),
		},
		None => true,
	};

	let tier = state.get::<u8>("tier");

	let player_name = match get_player(&player, &state.req_client).await {
		Ok(player) => player.name,
		Err(why) => {
			log::warn!("[{}]: {} => {:?}", file!(), line!(), why);
			String::from("unknown")
		},
	};

	let (description, amount) =
		match get_unfinished(&player, &mode, runtype, tier, &state.req_client).await {
			Ok(map_list) => {
				let description = if map_list.len() <= 10 {
					map_list.join("\n")
				} else {
					format!("{}\n...{} more", (map_list[0..10]).join("\n"), map_list.len() - 10)
				};

				let amount = format!(
					"{} uncompleted map{}",
					map_list.len(),
					if map_list.len() == 1 { "" } else { "s" }
				);
				(description, amount)
			},
			Err(why) => {
				log::warn!("[{}]: {} => {:?}", file!(), line!(), why);
				return Ok(Message(why.tldr));
			},
		};

	let embed = CreateEmbed::default()
		.colour(state.colour)
		.title(format!(
			"{} - {} {} {}",
			amount,
			&mode.to_fancy(),
			if runtype { "TP" } else { "PRO" },
			match tier {
				Some(tier) => format!("[T{}]", tier),
				None => String::new(),
			}
		))
		.description(if description.len() > 0 {
			description
		} else {
			String::from("You have no maps left to complete! Congrats! 🥳")
		})
		.footer(|f| f.text(format!("Player: {}", player_name)).icon_url(&state.icon))
		.to_owned();

	return Ok(Embed(embed));
}
