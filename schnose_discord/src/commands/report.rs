use {
	super::handle_err,
	crate::SchnoseError,
	log::{error, info},
	poise::{execute_modal, serenity_prelude::ChannelId, ApplicationContext, Modal},
	std::time::Duration,
};

#[derive(Debug, Modal)]
#[name = "Report Issues / Suggest changes"]
struct ReportModal {
	#[name = "Title"]
	#[placeholder = "title"]
	title: String,
	#[name = "Description"]
	#[placeholder = "Describe your issue / suggestion here."]
	#[paragraph]
	description: String,
}

/// Report issues with the bot or suggest changes!
#[poise::command(slash_command, on_error = "handle_err")]
pub async fn report(
	ctx: ApplicationContext<'_, crate::GlobalState, SchnoseError>,
) -> Result<(), SchnoseError> {
	let modal = ReportModal {
		title: String::from("<title>"),
		description: String::from("<description>"),
	};

	let modal = execute_modal(ctx, Some(modal), Some(Duration::from_secs(600))).await?;

	match modal {
		None => {
			ctx.say("Failed to submit.").await?;

			error!("Failed to submit report.");

			Ok(())
		}
		Some(ReportModal { title, description }) => {
			let channel = ChannelId(
				ctx.framework()
					.user_data()
					.await
					.config
					.report_channel_id,
			);
			channel
				.send_message(&ctx.serenity_context().http, |msg| {
					msg.embed(|e| {
						e.title(title)
							.description(description)
							.footer(|f| {
								f.text(format!(
									"User: {} | {}",
									ctx.author().tag(),
									chrono::Utc::now().format("%d/%m/%Y - %H:%M:%S")
								))
							})
					})
				})
				.await?;

			ctx.send(|reply| {
				reply.ephemeral(true);
				reply.content("Thanks for your submission!")
			})
			.await?;

			info!("Got a report.");

			Ok(())
		}
	}
}
