use crate::{utils::poise::CommandResult, Context};

mod data;

#[tracing::instrument(skip_all)]
#[poise::command(
    slash_command,
    prefix_command,
    discard_spare_arguments,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
pub async fn nortverse(ctx: Context<'_>) -> crate::Result<()> {
    Ok(())
}
