use poise::{serenity_prelude::CreateAttachment, CreateReply};
use serde::Deserialize;
use tracing::instrument;

use crate::{
    commands::LogCommands,
    utils::{
        poise::{CommandResult, ContextExt},
        Context,
    },
};

#[instrument(skip_all)]
#[poise::command(
    slash_command,
    prefix_command,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
pub async fn fox(ctx: Context<'_>) -> CommandResult {
    ctx.log_command().await;

    #[derive(Deserialize)]
    struct ApiResponse {
        image: String,
    }

    let json: ApiResponse = reqwest::get("https://randomfox.ca/floof/")
        .await?
        .json::<ApiResponse>()
        .await?;

    let attachment = CreateAttachment::url(&ctx, &json.image).await?;
    let reply = CreateReply::default()
        .content("fox courtesy of [randomfox.ca](<https://randomfox.ca/>)")
        .attachment(attachment)
        .reply(true);

    ctx.send_ext(reply).await?;

    Ok(())
}
