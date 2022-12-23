use {
	crate::{GlobalState, prelude::PaginationData},
	log::{trace, info, warn, error},
	serenity::{
		prelude::Context,
		model::prelude::interaction::{
			message_component::MessageComponentInteraction, InteractionResponseType,
		},
	},
};

pub(crate) async fn handle(
	_global_state: &GlobalState,
	ctx: Context,
	component: &MessageComponentInteraction,
) -> anyhow::Result<()> {
	component
		.create_interaction_response(&ctx.http, |response| {
			response
				.kind(InteractionResponseType::UpdateMessage)
				.interaction_response_data(|response| response.content(""))
		})
		.await?;

	let message = component.get_interaction_response(&ctx.http).await?;

	let Some(interaction) = message.interaction else {
		// fuck anyhow
		error!("Failed to get original interaction.");
		return Ok(());
	};

	let interaction_id = *interaction.id.as_u64();

	let mut global_data = ctx.data.write().await;
	let global_data = global_data
		.get_mut::<PaginationData>()
		.expect("Buttons should never be created if there is no global data.");

	global_data.keys().for_each(|k| println!("{k}"));

	let Some(current_data) = global_data.get_mut(&interaction_id) else {
		trace!("no data, got deleted probably");
		return Ok(());
	};

	let current_index = current_data.current_index;
	let offset_back =
		// go to last page if `<` is pressed on the first page, otherwise go back 1 page
		if current_index == 0 { current_data.embed_list.len() - 1 } else { current_index - 1 };
	let offset_forward =
		// go to first page if `>` is pressed on the last page, otherwise go forward 1 page
		if current_index == current_data.embed_list.len() - 1 { 0 } else { current_index + 1 };

	let new_embed = match component.data.custom_id.as_str() {
		"go-back" => {
			current_data.current_index = offset_back;
			current_data.embed_list[offset_back].clone()
		},
		"go-forward" => {
			current_data.current_index = offset_forward;
			current_data.embed_list[offset_forward].clone()
		},
		unknown_id => {
			warn!("Encountered unknown component_id: {}", unknown_id);
			return Ok(());
		},
	};

	info!("we flipped");

	// update message
	if let Err(why) = component
		.edit_original_interaction_response(&ctx.http, |response| response.set_embed(new_embed))
		.await
	{
		warn!("Failed to update message: {:?}", why);
	}

	Ok(())
}
