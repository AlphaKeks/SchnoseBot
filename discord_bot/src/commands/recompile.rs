use crate::{error::Error, Context, State};

#[poise::command(
	prefix_command,
	on_error = "Error::handle_command",
	owners_only,
	global_cooldown = 120
)]
pub async fn recompile(ctx: Context<'_>, clean: Option<String>) -> Result<(), Error> {
	ctx.defer().await?;

	let msg_handle = ctx
		.say("Preparing for compilation...")
		.await?;

	let old_content = if matches!(dbg!(clean).as_deref(), Some("clean")) {
		let old_content = &msg_handle.message().await?.content;
		let new_content = format!("{old_content}\nCleaning build directory...");
		msg_handle
			.edit(ctx, |msg| msg.content(&new_content))
			.await?;

		crate::process::cargo_clean(ctx.config())?;

		new_content
	} else {
		msg_handle
			.message()
			.await?
			.content
			.clone()
	};

	let new_content = format!("{old_content}\nCompiling...");

	msg_handle
		.edit(ctx, |msg| msg.content(&new_content))
		.await?;

	let (stdout, stderr) = match crate::process::cargo_build(ctx.config()) {
		Ok((stdout, stderr)) => (
			stdout.lines().last().map(|line| {
				line.strip_prefix("    ")
					.map(String::from)
					.unwrap_or_default()
			}),
			stderr.lines().last().map(|line| {
				line.strip_prefix("    ")
					.map(String::from)
					.unwrap_or_default()
			}),
		),
		Err(why) => (None, Some(why.to_string())),
	};

	let output = match (stdout, stderr) {
		(Some(stdout), Some(stderr)) => format!(
			r#"
{new_content}
```
{stdout}
```
```
{stderr}
```
            "#
		),
		(Some(stdout), None) => format!(
			r#"
{new_content}
```
{stdout}
```
            "#
		),
		(None, Some(stderr)) => format!(
			r#"
{new_content}
```
{stderr}
```
            "#
		),
		(None, None) => String::new(),
	};

	msg_handle
		.edit(ctx, |msg| msg.content(output))
		.await?;

	Ok(())
}
