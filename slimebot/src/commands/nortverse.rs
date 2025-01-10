use crate::{
    utils::poise::{CommandResult, ContextExt},
    Context,
};

pub use slimebot_nortverse::Error;
use tracing_unwrap::ResultExt;

#[derive(Debug, Clone)]
pub struct Nortverse(slimebot_nortverse::Nortverse);

impl Nortverse {
    pub fn from_database(db: &mongodb::Database) -> Self {
        Self(slimebot_nortverse::Nortverse::from_database(db))
    }

    #[tracing::instrument(skip_all)]
    pub async fn subscribe_action(
        &self,
        cache: std::sync::Arc<poise::serenity_prelude::Cache>,
        http: std::sync::Arc<poise::serenity_prelude::Http>,
    ) -> CommandResult {
        tracing::info!("checking for new comic");

        try {
            let (comic, updated, old_slug) = self.0.refresh_latest().await?;

            if updated {
                tracing::info!(comic.slug = comic.slug(), old.slug = ?old_slug, "new comic found");

                let message = {
                    comic
                        .builder()
                        .in_guild(false)
                        .include_date(false)
                        .subscribed(true)
                        .build_message(&http)
                        .await?
                };

                for subscriber in self.0.subscribers().await? {
                    let message = message.clone();
                    let cache = cache.clone();
                    let http = http.clone();

                    tracing::trace!(user.id = %subscriber, "messaging subscriber");

                    use crate::utils::serenity::UserIdExt;

                    tokio::spawn(async move {
                        subscriber
                            .dm_ext((&cache, http.as_ref()), message.clone())
                            .await
                            .expect_or_log("failed to send message, skipping...");
                    });
                }
            } else {
                tracing::trace!("no new comic found")
            }
        }
    }

    #[tracing::instrument(skip_all)]
    pub fn subscribe_task(
        self,
        cache: std::sync::Arc<poise::serenity_prelude::Cache>,
        http: std::sync::Arc<poise::serenity_prelude::Http>,
    ) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_mins(60));

            loop {
                interval.tick().await;

                self.subscribe_action(cache.clone(), http.clone())
                    .await
                    .expect_or_log("failed to run subscribe task");
            }
        });
    }
}

impl std::ops::Deref for Nortverse {
    type Target = slimebot_nortverse::Nortverse;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

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
