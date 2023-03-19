use {
	crate::{
		error::{Error, Result},
		Context, GlobalState, State,
	},
	chrono::Utc,
	log::trace,
	poise::{
		execute_modal,
		serenity_prelude::{CacheHttp, ChannelId},
		ApplicationContext, Modal,
	},
	std::time::Duration,
};

/// Report issues/bugs with the bot or suggest changes.
///
/// This command will open a `Modal` where you can describe an issue you had with the bot. The \
/// contents of that `Modal` will be sent to me (AlphaKeks) for review. The more info you provide, \
/// the better. Timestamps, screenshots, detailed description etc. make it much easier to fix bugs.
#[poise::command(slash_command, on_error = "Error::handle_command")]
pub async fn report(ctx: ApplicationContext<'_, GlobalState, Error>) -> Result<()> {
	trace!("[/report ({})]", ctx.author().tag());

	let Some(modal) = execute_modal(ctx, Some(Report::default()), Some(Duration::from_secs(300))).await? else {
		// User didn't submit modal in time.
		return Ok(());
	};

	let ctx = Context::from(ctx);

	ChannelId(ctx.config().report_channel)
		.send_message(ctx.serenity_context().http(), |msg| {
			msg.embed(|e| {
				e.title(modal.title)
					.color(ctx.color())
					.description(modal.description)
					.thumbnail(
						ctx.author()
							.avatar_url()
							.unwrap_or_else(|| ctx.author().default_avatar_url()),
					)
					.footer(|f| {
						f.text(format!(
							"User: {} | {}",
							ctx.author().tag(),
							Utc::now().format("%d/%m/%Y - %H:%M:%S")
						))
					})
			})
		})
		.await?;

	ctx.send(|reply| {
		reply
			.ephemeral(true)
			.content("Thanks for your submission!")
	})
	.await?;

	Ok(())
}

#[derive(Debug, Default, Modal)]
#[name = "Report Issue / Suggest change"]
struct Report {
	#[name = "Title"]
	#[placeholder = "<title>"]
	title: String,

	#[name = "Description"]
	#[placeholder = "Describe your issue here. Please provide Screenshots if you can."]
	#[paragraph]
	description: String,
}
