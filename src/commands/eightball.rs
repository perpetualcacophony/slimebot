use tracing::instrument;

use crate::utils::{poise::CommandResult, Context};

mod core;
use core::ANSWERS;

/// posts a response from the magic 8-ball
#[instrument(skip_all)]
#[poise::command(
    slash_command,
    prefix_command,
    rename = "8ball",
    discard_spare_arguments,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
pub async fn eightball(ctx: Context<'_>) -> crate::Result<()> {
    _eightball(ctx).await?;
    Ok(())
}

async fn _eightball(ctx: Context<'_>) -> CommandResult {
    use rand::prelude::thread_rng;

    let answer = ANSWERS.get(&mut thread_rng());
    ctx.reply(answer).await?;

    Ok(())
}
