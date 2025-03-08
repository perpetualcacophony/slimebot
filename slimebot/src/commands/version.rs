use tracing::instrument;

use crate::utils::poise::CommandResult;
use crate::Context;
use crate::{built_info, utils::poise::ContextExt, Result};

/// displays the bot's current version
#[instrument(skip_all)]
#[poise::command(
    slash_command,
    prefix_command,
    discard_spare_arguments,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
pub async fn version(ctx: Context<'_>) -> Result<()> {
    _version(ctx).await?;
    Ok(())
}

async fn _version(ctx: Context<'_>) -> CommandResult {
    let build = if built_info::DEBUG {
        let branch = built_info::GIT_HEAD_REF
            .map(|s| {
                s.split('/')
                    .next_back()
                    .expect("head ref should have slashes")
            })
            .unwrap_or("DETACHED");

        format!(
            "development branch {} (`{}`)",
            branch,
            built_info::GIT_COMMIT_HASH_SHORT.expect("should be built with a git repo")
        )
    } else {
        format!("release {}", built_info::PKG_VERSION)
    };

    ctx.reply_ext(build).await?;

    Ok(())
}
