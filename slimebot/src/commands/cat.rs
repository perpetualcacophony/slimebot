use poise::{serenity_prelude::CreateAttachment, CreateReply};
use tracing::instrument;

use crate::{
    commands::LogCommands,
    utils::{
        poise::{CommandResult, ContextExt},
        Context,
    },
};

/// posts random cat media from cataas.com
#[instrument(skip_all)]
#[poise::command(
    slash_command,
    prefix_command,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
pub async fn cat(ctx: Context<'_>, #[flag] gif: bool) -> crate::Result<()> {
    _cat(ctx, gif).await?;
    Ok(())
}

async fn _cat(ctx: Context<'_>, gif: bool) -> CommandResult {
    ctx.log_command().await;

    let (url, filename) = if gif {
        ("https://cataas.com/cat/gif", "cat.gif")
    } else {
        ("https://cataas.com/cat", "cat.jpg") // i don't know why this works
                                              // but asserting all images, even png ones, as .jpg is... fine, i guess?
                                              // thanks discord
    };

    let response = reqwest::get(url).await?;

    let bytes = response.bytes().await?;

    let attachment = CreateAttachment::bytes(bytes, filename);
    let reply = CreateReply::default()
        .content("cat courtesy of [cataas.com](<https://cataas.com/>)")
        .attachment(attachment)
        .reply(true);

    ctx.send_ext(reply).await?;

    Ok(())
}
