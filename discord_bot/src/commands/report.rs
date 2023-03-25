use {
	crate::{
		error::{Error, Result},
		Context, GlobalState, State,
	},
	chrono::Utc,
	poise::{
		execute_modal,
		serenity_prelude::{CacheHttp, ChannelId},
		ApplicationContext, Modal,
	},
	std::time::Duration,
};

/// Report issues/bugs with the bot or suggest changes.
///
/// This command will open a pop-up where you can submit bug reports / suggestions for the bot (in \
/// case you don't like GitHub issues). The information you put in there will be sent to a channel \
/// that can be specified in the bot's config file. If you use my instance of the bot, that \
/// channel is a private channel on my Discord server that only I and a few admins have access to.
#[tracing::instrument(skip(ctx), fields(user = ctx.author().tag()))]
#[poise::command(slash_command, on_error = "Error::handle_command")]
pub async fn report(ctx: ApplicationContext<'_, GlobalState, Error>) -> Result<()> {
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
