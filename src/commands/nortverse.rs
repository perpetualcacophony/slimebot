use crate::{
    utils::poise::{CommandResult, ContextExt},
    Context,
};
use poise::serenity_prelude as serenity;
use serenity::futures::stream;

mod data;

mod error;
pub use error::NortverseError as Error;

mod comic;

mod client;
pub use client::Nortverse;

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
    use stream::{StreamExt, TryStreamExt};

    let result: CommandResult = try {
        ctx.defer_or_broadcast().await?;

        let comic = ctx.data().nortverse().refresh_latest().await?.0;

        let builder = poise::CreateReply::default().reply(true).content(format!(
            "## {title}\nPosted {date}",
            title = comic.title(),
            date = comic.date()
        ));

        let attachments = stream::iter(comic.images())
            .then(|url| serenity::CreateAttachment::url(ctx.http(), url.as_str()))
            .try_collect::<Vec<_>>()
            .await?
            .into_iter();

        let response = attachments.fold(builder, |builder, mut attachment| {
            if ctx.guild_id().is_some() {
                attachment.filename = format!("SPOILER_{original}", original = attachment.filename);
            }

            builder.attachment(attachment)
        });

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
        ctx.defer_or_broadcast().await?;

        ctx.data()
            .nortverse()
            .add_subscriber(ctx.author().id)
            .await?;

        ctx.reply_ephemeral("you'll be notified whenever a new comic is posted!")
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
        ctx.defer_or_broadcast().await?;

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
    use stream::{StreamExt, TryStreamExt};

    let result: CommandResult = try {
        ctx.defer_or_broadcast().await?;

        let nortverse = ctx.data().nortverse();

        let comic = nortverse.random_comic().await?;

        let builder = poise::CreateReply::default().reply(true).content(format!(
            "## {title}\nPosted {date}",
            title = comic.title(),
            date = comic.date()
        ));

        let attachments = stream::iter(comic.images())
            .then(|url| serenity::CreateAttachment::url(ctx.http(), url.as_str()))
            .try_collect::<Vec<_>>()
            .await?
            .into_iter();

        let response = attachments.fold(builder, |builder, mut attachment| {
            if ctx.guild_id().is_some() {
                attachment.filename = format!("SPOILER_{original}", original = attachment.filename);
            }

            builder.attachment(attachment)
        });

        ctx.send_ext(response).await?;
    };

    result?;
    Ok(())
}
