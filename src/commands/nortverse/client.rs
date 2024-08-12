use std::sync::Arc;

use poise::serenity_prelude as serenity;
use tracing_unwrap::ResultExt;

use crate::utils::poise::CommandResult;

use super::{
    comic::ComicPage,
    data::{self, NortverseDataAsync},
    Error,
};

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug)]
pub struct Nortverse<Data = data::MongoDb> {
    data: std::sync::Arc<tokio::sync::RwLock<Data>>,
    client: reqwest::Client,
}

impl Nortverse {
    pub fn from_database(db: &mongodb::Database) -> Self {
        Self::new(data::MongoDb::from_database(db))
    }

    #[tracing::instrument(skip_all)]
    pub async fn subscribe_action(
        &self,
        cache: Arc<serenity::Cache>,
        http: Arc<serenity::Http>,
    ) -> CommandResult {
        tracing::info!("checking for new comic");

        try {
            let (comic, updated, old_slug) = self.refresh_latest().await?;

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

                for subscriber in self.subscribers().await? {
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
    pub fn subscribe_task(self, cache: Arc<serenity::Cache>, http: Arc<serenity::Http>) {
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

impl<T> Clone for Nortverse<T> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            client: self.client.clone(),
        }
    }
}

impl<Data> Nortverse<Data> {
    fn new(data: Data) -> Self {
        Self {
            data: std::sync::Arc::new(tokio::sync::RwLock::new(data)),
            client: reqwest::Client::new(),
        }
    }

    async fn data(&self) -> tokio::sync::RwLockReadGuard<Data> {
        self.data.read().await
    }

    async fn data_mut(&self) -> tokio::sync::RwLockWriteGuard<Data> {
        self.data.write().await
    }

    fn client(&self) -> &reqwest::Client {
        &self.client
    }

    pub async fn random_comic(&self) -> Result<ComicPage> {
        Ok(ComicPage::random(self.client()).await?)
    }

    pub async fn latest_comic(&self) -> Result<ComicPage> {
        Ok(ComicPage::from_homepage(self.client()).await?)
    }
}

impl<Data> Nortverse<Data>
where
    Data: NortverseDataAsync,
{
    #[tracing::instrument(skip_all)]
    pub async fn refresh_latest(&self) -> Result<(ComicPage, bool, Option<String>)> {
        let latest = self.latest_comic().await?;

        let data_slug = {
            let data = self.data().await;
            let data_slug = data.latest_slug().await.map_err(Error::data)?;
            data_slug.map(|as_ref| as_ref.as_ref().to_owned())
        };

        let updated = Some(latest.slug()) != data_slug.as_deref();

        if updated {
            self.data_mut()
                .await
                .set_latest(latest.slug().to_owned())
                .await
                .map_err(Error::data)?;

            tracing::info!(slug = %latest.slug(), "updated latest comic")
        }

        Ok((latest, updated, data_slug))
    }

    #[tracing::instrument(skip_all, fields(id))]
    pub async fn add_subscriber(&self, id: serenity::UserId) -> Result<()> {
        let mut data = self.data_mut().await;

        if data.contains_subscriber(&id).await.map_err(Error::data)? {
            tracing::warn!(user.id = %id, "user already subscribed");

            Err(Error::already_subscribed(id))
        } else {
            data.add_subscriber(id).await.map_err(Error::data)?;

            tracing::info!(user.id = %id, "added subscriber");

            Ok(())
        }
    }

    pub async fn remove_subscriber(&self, id: serenity::UserId) -> Result<()> {
        let mut data = self.data_mut().await;

        if data.contains_subscriber(&id).await.map_err(Error::data)? {
            data.remove_subscriber(id).await.map_err(Error::data)?;

            tracing::info!(user.id = %id, "removed subscriber");

            Ok(())
        } else {
            tracing::warn!(user.id = %id, "user not subscribed");

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
