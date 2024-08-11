use crate::{
    utils::poise::{CommandResult, ContextExt},
    Context,
};
use poise::serenity_prelude as serenity;
use serenity::futures::stream;

mod data;
use data::NortverseDataAsync;

mod error;
pub use error::NortverseError as Error;

mod comic;
use comic::ComicPage;

type Result<T, E = Error> = std::result::Result<T, E>;

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

#[derive(Debug)]
pub struct Nortverse<Data = data::MongoDb> {
    data: std::sync::Arc<tokio::sync::RwLock<Data>>,
}

impl Nortverse {
    pub fn from_database(db: &mongodb::Database) -> Self {
        Self {
            data: std::sync::Arc::new(tokio::sync::RwLock::new(data::MongoDb::from_database(db))),
        }
    }

    pub async fn subscribe_action(&self, cache_http: &impl serenity::CacheHttp) -> CommandResult {
        try {
            let (comic, updated) = self.refresh_latest().await?;

            if updated {
                let message = {
                    use stream::{StreamExt, TryStreamExt};

                    let attachments = stream::iter(comic.images())
                        .then(|url| {
                            serenity::CreateAttachment::url(cache_http.http(), url.as_str())
                        })
                        .try_collect::<Vec<_>>()
                        .await?;

                    serenity::CreateMessage::new()
                        .content(format!(
                            "new comic!\n## {title}\n(`..nortverse unsubscribe` to unsubscribe)",
                            title = comic.title(),
                        ))
                        .add_files(attachments)
                };

                for subscriber in self.subscribers().await? {
                    subscriber.dm(cache_http, message.clone()).await?;
                }
            }
        }
    }

    pub fn subscribe_task(self, cache_http: impl serenity::CacheHttp + 'static) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_hours(1));

            loop {
                interval.tick().await;

                // todo: handle this error somewhere
                self.subscribe_action(&cache_http).await;
            }
        });
    }
}

impl<T> Clone for Nortverse<T> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
        }
    }
}

impl<Data> Nortverse<Data> {
    async fn data(&self) -> tokio::sync::RwLockReadGuard<Data> {
        self.data.read().await
    }

    async fn data_mut(&self) -> tokio::sync::RwLockWriteGuard<Data> {
        self.data.write().await
    }

    const HOST: &str = "nortverse.com";
    const COMICS_PATH: &str = "comic";
    const RANDOM_PATH: &str = "random";

    fn comic_url(slug: &str) -> reqwest::Url {
        reqwest::Url::parse(&format!(
            "https://{host}/{path}/{slug}",
            host = Self::HOST,
            path = Self::COMICS_PATH
        ))
        .expect("url should be well-formed")
    }

    async fn random_comic() -> Result<ComicPage> {
        let seed: u64 = rand::random();

        let response = reqwest::get(format!(
            "https://{host}/{path}?r={seed}",
            host = Self::HOST,
            path = Self::RANDOM_PATH
        ))
        .await?;

        let slug = response
            .url()
            .path_segments()
            .expect("should have path")
            .nth(1)
            .expect("should have 2nd item");

        Ok(ComicPage::from_slug(slug).await?)
    }
}

impl<Data> Nortverse<Data>
where
    Data: NortverseDataAsync,
{
    async fn refresh_latest(&self) -> Result<(ComicPage, bool)> {
        let latest = ComicPage::from_homepage().await?;

        let data_slug = {
            let data = self.data().await;
            let data_slug = data.latest_slug().await.map_err(Error::data)?;
            data_slug.map(|as_ref| as_ref.as_ref().to_owned())
        };

        let updated = Some(latest.slug()) == data_slug.as_deref();

        if updated {
            self.data_mut()
                .await
                .set_latest(latest.slug().to_owned())
                .await
                .map_err(Error::data)?;
        }

        Ok((latest, updated))
    }

    async fn add_subscriber(&self, id: serenity::UserId) -> Result<()> {
        let mut data = self.data_mut().await;

        if data.contains_subscriber(&id).await.map_err(Error::data)? {
            Err(Error::already_subscribed(id))
        } else {
            data.add_subscriber(id).await.map_err(Error::data)
        }
    }

    async fn remove_subscriber(&self, id: serenity::UserId) -> Result<()> {
        let mut data = self.data_mut().await;

        if data.contains_subscriber(&id).await.map_err(Error::data)? {
            data.remove_subscriber(id).await.map_err(Error::data)
        } else {
            Err(Error::not_subscribed(id))
        }
    }

    async fn subscribers(&self) -> Result<impl Iterator<Item = serenity::UserId>> {
        let data = self.data().await;

        let vec: Vec<serenity::UserId> = data
            .subscribers()
            .await
            .map_err(Error::data)?
            .into_iter()
            .collect();

        Ok(vec.into_iter())
    }
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

        let comic = Nortverse::<()>::random_comic().await?;

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
