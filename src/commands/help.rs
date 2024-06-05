use tracing::instrument;

use crate::{
    errors::SendMessageError,
    utils::{poise::CommandResult, Context},
};

// displays command help text
#[instrument(skip_all)]
#[poise::command(
    slash_command,
    prefix_command,
    discard_spare_arguments,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "specific command to display help for"] command: Option<String>,
) -> CommandResult {
    poise::builtins::help(
        ctx,
        command.as_deref(),
        poise::builtins::HelpConfiguration::default(),
    )
    .await
    .map_err(SendMessageError::from)?;

    Ok(())
}
