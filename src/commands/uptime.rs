use tracing::instrument;

use crate::{
    commands::LogCommands,
    utils::{
        format_duration::FormatDuration,
        poise::{CommandResult, ContextExt},
        Context,
    },
    Result,
};

#[instrument(skip(ctx))]
#[poise::command(prefix_command, required_bot_permissions = "SEND_MESSAGES")]
pub async fn uptime(ctx: Context<'_>) -> Result<()> {
    _uptime(ctx).await?;
    Ok(())
}

async fn _uptime(ctx: Context<'_>) -> CommandResult {
    ctx.log_command().await;

    let started = ctx.data().started;
    let uptime = chrono::Utc::now() - started;

    ctx.reply_ext(format!(
        "uptime: {} (since {})",
        uptime.format_full(),
        started.format("%Y-%m-%d %H:%M UTC")
    ))
    .await?;

    Ok(())
}
