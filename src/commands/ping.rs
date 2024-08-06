use crate::{
    commands::LogCommands,
    utils::{
        poise::{CommandResult, ContextExt},
        Context,
    },
    Result,
};
use tracing::instrument;

/// prompts the bot for a response
#[instrument(skip_all)]
#[poise::command(
    slash_command,
    prefix_command,
    discard_spare_arguments,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
pub async fn ping(ctx: Context<'_>) -> Result<()> {
    ctx.log_command().await;
    _ping(ctx).await?;
    Ok(())
}

async fn _ping(ctx: Context<'_>) -> CommandResult {
    let ping = ctx.ping().await.as_millis();
    if ping == 0 {
        ctx.reply_ext("pong! (please try again later to display latency)")
            .await?;
    } else {
        ctx.reply_ext(format!("pong! ({}ms)", ping)).await?;
    }

    Ok(())
}

/// alternate version of ping command
#[instrument(skip_all)]
#[poise::command(
    slash_command,
    prefix_command,
    hide_in_help,
    discard_spare_arguments,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
pub async fn pong(ctx: Context<'_>) -> Result<()> {
    ctx.log_command().await;
    _pong(ctx).await?;
    Ok(())
}

async fn _pong(ctx: Context<'_>) -> CommandResult {
    let ping = ctx.ping().await.as_millis();
    if ping == 0 {
        ctx.reply_ext("ping! (please try again later to display latency)")
            .await?;
    } else {
        ctx.reply_ext(format!("ping! ({}ms)", ping)).await?;
    }

    Ok(())
}
