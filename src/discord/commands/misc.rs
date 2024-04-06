use super::CommandResult;
use crate::functions::misc;
use crate::Context;
use tracing::instrument;

#[instrument(skip_all)]
#[poise::command(
    slash_command,
    prefix_command,
    rename = "8ball",
    discard_spare_arguments,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
pub async fn eightball(ctx: Context<'_>) -> CommandResult {
    use rand::prelude::thread_rng;

    let answer = misc::ANSWERS.get(&mut thread_rng());
    ctx.reply(answer).await?;

    Ok(())
}
