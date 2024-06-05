use poise::CreateReply;
use tracing::instrument;

use crate::utils::{
    poise::{CommandResult, ContextExt},
    Context,
};

#[instrument(skip(ctx))]
#[poise::command(prefix_command, required_bot_permissions = "SEND_MESSAGES")]
pub async fn banban(ctx: Context<'_>) -> CommandResult {
    if ctx.author().id == 497014954935713802 {
        super::ban::core::joke_ban(
            ctx,
            ctx.author(),
            966519580266737715,
            "get banbanned lol".to_string(),
        )
        .await?;
    } else {
        ctx.send_ext(CreateReply::default().content("https://files.catbox.moe/jm6sr9.png"))
            .await?;
    }

    Ok(())
}
