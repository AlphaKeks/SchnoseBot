use {
	crate::{
		prelude::InteractionResponseData, commands, GlobalState,
		events::interactions::InteractionState,
	},
	log::{trace, warn},
	serenity::{
		prelude::Context,
		model::prelude::interaction::application_command::ApplicationCommandInteraction,
	},
};

pub(crate) async fn handle(
	global_state: &GlobalState,
	ctx: Context,
	interaction: &ApplicationCommandInteraction,
) -> anyhow::Result<Option<u64>> {
	let command_name = interaction.data.name.as_str();

	trace!("Received slash command: `{}`", command_name);

	let mut interaction_state = InteractionState::new(
		&ctx.http,
		interaction,
		&global_state.db,
		&global_state.req_client,
		global_state.colour,
		&global_state.icon,
	);

	let response = match command_name {
		"apistatus" => commands::apistatus::execute(&mut interaction_state).await,
		"bpb" => commands::bpb::execute(&mut interaction_state).await,
		"bwr" => commands::bwr::execute(&mut interaction_state).await,
		"bmaptop" => commands::bmaptop::execute(&mut interaction_state).await,
		"db" => commands::db::execute(&mut interaction_state).await,
		"invite" => commands::invite::execute(&mut interaction_state).await,
		"map" => commands::map::execute(&mut interaction_state).await,
		"maptop" => commands::maptop::execute(&mut interaction_state).await,
		"mode" => commands::mode::execute(&mut interaction_state).await,
		"nocrouch" => commands::nocrouch::execute(&interaction_state).await,
		"pb" => commands::pb::execute(&mut interaction_state).await,
		"ping" => commands::ping::execute().await,
		"profile" => commands::profile::execute(&mut interaction_state).await,
		"random" => commands::random::execute(&interaction_state).await,
		"recent" => commands::recent::execute(&mut interaction_state).await,
		"setsteam" => commands::setsteam::execute(&mut interaction_state).await,
		"unfinished" => commands::unfinished::execute(&mut interaction_state).await,
		"wr" => commands::wr::execute(&mut interaction_state).await,
		unknown_command => {
			warn!("Encountered unknown slash command `{}`.", unknown_command);
			return Ok(None);
		},
	};

	// check if there is the need to spawn a garbage collector
	let spawn_gc = matches!(response, Ok(InteractionResponseData::Pagination(_)));

	interaction_state.reply(ctx.data, response).await?;

	if spawn_gc {
		Ok(Some(*interaction.id.as_u64()))
	} else {
		Ok(None)
	}
}
