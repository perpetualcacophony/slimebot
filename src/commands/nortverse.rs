use crate::{
    utils::poise::{CommandResult, ContextExt},
    Context,
};

mod data;

mod error;
pub use error::NortverseError as Error;

mod comic;

mod client;
pub use client::Nortverse;

mod response;

#[tracing::instrument(skip_all)]
#[poise::command(
    slash_command,
    prefix_command,
    discard_spare_arguments,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL",
    subcommands("latest", "subscribe", "unsubscribe", "random")
)]
pub async fn nortverse(ctx: Context<'_>) -> crate::Result<()> {
    latest_inner(ctx).await
}

#[tracing::instrument(skip_all)]
#[poise::command(
    slash_command,
    prefix_command,
    discard_spare_arguments,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
pub async fn latest(ctx: Context<'_>) -> crate::Result<()> {
    latest_inner(ctx).await
}

async fn latest_inner(ctx: Context<'_>) -> crate::Result<()> {
    let result: CommandResult = try {
        let _broadcast = ctx.defer_or_broadcast().await?;

        let response = ctx
            .data()
            .nortverse()
            .latest_comic()
            .await?
            .builder()
            .in_guild(ctx.guild_id().is_some())
            .build_reply(ctx.http())
            .await?
            .reply(true);

        ctx.send_ext(response).await?;
    };

    result?;
    Ok(())
}

#[tracing::instrument(skip_all)]
#[poise::command(
    slash_command,
    prefix_command,
    discard_spare_arguments,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
pub async fn subscribe(ctx: Context<'_>) -> crate::Result<()> {
    let result: CommandResult = try {
        let _broadcast = ctx.defer_or_broadcast().await?;

        ctx.data()
            .nortverse()
            .add_subscriber(ctx.author().id)
            .await?;

        ctx.reply_ephemeral("you'll be notified whenever a new comic is posted!\n`..nortverse unsubscribe` to unsubscribe")
            .await?;
    };

    result?;
    Ok(())
}

#[tracing::instrument(skip_all)]
#[poise::command(
    slash_command,
    prefix_command,
    discard_spare_arguments,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
pub async fn unsubscribe(ctx: Context<'_>) -> crate::Result<()> {
    let result: CommandResult = try {
        let _broadcast = ctx.defer_or_broadcast().await?;

        ctx.data()
            .nortverse()
            .remove_subscriber(ctx.author().id)
            .await?;

        ctx.reply_ephemeral("you will no longer be notified for new comics.")
            .await?;
    };

    result?;
    Ok(())
}

#[tracing::instrument(skip_all)]
#[poise::command(
    slash_command,
    prefix_command,
    discard_spare_arguments,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
pub async fn random(ctx: Context<'_>) -> crate::Result<()> {
    let result: CommandResult = try {
        let _broadcast = ctx.defer_or_broadcast().await?;

        let nortverse = ctx.data().nortverse();

        let response = nortverse
            .random_comic()
            .await?
            .builder()
            .in_guild(ctx.guild_id().is_some())
            .build_reply(ctx.http())
            .await?
            .reply(true);

        ctx.send_ext(response).await?;
    };

    result?;
    Ok(())
}
