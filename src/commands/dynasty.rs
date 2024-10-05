use poise::{
    serenity_prelude::futures::{stream, StreamExt, TryStreamExt},
    CreateReply,
};

use poise::serenity_prelude as serenity;

use crate::utils::{
    poise::{CommandResult, ContextExt},
    Context,
};

mod client;
use client::Client;

mod series;
use series::Series;

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

        let dynasty = ctx.data().dynasty();

        let series = dynasty
            .series(&"the_guy_she_was_interested_in_wasnt_a_guy_at_all")?
            .await?;

        let chapter = series
            .chapters()
            .last()
            .expect("should have at least one chapter");
        let chapter = dynasty.chapter(&chapter.slug())?.await?;

        let attachments = stream::iter(chapter.pages())
            .then(|page| async move {
                serenity::CreateAttachment::url(ctx.http(), page.image_url().as_str()).await
            })
            .try_collect::<Vec<_>>()
            .await?;

        let builder = CreateReply::default().content("tgswii");

        let builder = attachments
            .into_iter()
            .fold(builder, |builder, attachment| {
                builder.attachment(attachment)
            });

        ctx.send_ext(builder).await?;
    };

    result?;

    Ok(())
}
