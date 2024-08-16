use crate::utils::{poise::CommandResult, Context};

#[tracing::instrument(skip_all)]
#[poise::command(
    slash_command,
    prefix_command,
    discard_spare_arguments,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
pub async fn dynasty(ctx: Context<'_>) -> crate::Result<()> {
    let result: CommandResult = try {
        let _broadcast = ctx.defer_or_broadcast().await?;
    };

    result?;

    Ok(())
}
