use tracing::instrument;

use crate::utils::{poise::CommandResult, Context};

mod core;
use core::DiceRoll;

#[instrument(skip_all)]
#[poise::command(
    slash_command,
    prefix_command,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
pub async fn roll(ctx: Context<'_>, #[rest] text: String) -> CommandResult {
    let roll = DiceRoll::parse(&text)?;
    let result = roll.result();

    ctx.reply(result.to_string()).await?;

    Ok(())
}

#[instrument(skip_all)]
#[poise::command(
    slash_command,
    prefix_command,
    discard_spare_arguments,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
pub async fn d20(ctx: Context<'_>) -> CommandResult {
    let _typing = ctx.defer_or_broadcast().await?;

    let roll = DiceRoll::new(1, 20, 0).expect("hard-coded");
    let result = roll.result();

    ctx.reply(format!("**{result}**")).await?;

    Ok(())
}
