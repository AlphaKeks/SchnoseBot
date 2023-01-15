use {
	crate::GlobalState,
	log::info,
	poise::serenity_prelude::{Activity, Context, Ready, OnlineStatus::Online},
};

pub async fn handle(
	ctx: &Context,
	_global_state: &GlobalState,
	ready: &Ready,
) -> Result<(), crate::SchnoseError> {
	info!("Connected to Discord as `{}`.", ready.user.tag());

	ctx.set_presence(Some(Activity::playing("kz_epiphany_v2")), Online)
		.await;

	Ok(())
}
