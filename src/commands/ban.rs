use crate::utils::{poise::CommandResult, Context, Result};
use poise::serenity_prelude::User;
use tracing::instrument;

pub mod core;
use core::joke_ban;

#[instrument(skip(ctx, user))]
#[poise::command(prefix_command, required_bot_permissions = "SEND_MESSAGES")]
pub async fn ban(ctx: Context<'_>, user: User, reason: Option<String>) -> Result<()> {
    _ban(ctx, user, reason).await?;
    Ok(())
}

async fn _ban(ctx: Context<'_>, user: User, reason: Option<String>) -> CommandResult {
    if ctx.author().id == 497014954935713802 || user.id == 966519580266737715 {
        joke_ban(ctx, ctx.author(), 966519580266737715, "sike".to_string()).await?;
    } else {
        joke_ban(ctx, &user, ctx.author().id.get(), reason).await?;
    }

    Ok(())
}
