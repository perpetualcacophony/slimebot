use poise::serenity_prelude as serenity;

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

    pub async fn subscribe_action(&self, cache_http: &impl serenity::CacheHttp) -> CommandResult {
        try {
            let (comic, updated) = self.refresh_latest().await?;

            if updated {
                let message = {
                    comic
                        .builder()
                        .in_guild(false)
                        .include_date(false)
                        .subscribed(true)
                };

                for subscriber in self.subscribers().await? {
                    subscriber
                        .dm(cache_http, message.build_message(cache_http.http()).await?)
                        .await?;
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
    pub async fn refresh_latest(&self) -> Result<(ComicPage, bool)> {
        let latest = self.latest_comic().await?;

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

    pub async fn add_subscriber(&self, id: serenity::UserId) -> Result<()> {
        let mut data = self.data_mut().await;

        if data.contains_subscriber(&id).await.map_err(Error::data)? {
            Err(Error::already_subscribed(id))
        } else {
            data.add_subscriber(id).await.map_err(Error::data)
        }
    }

    pub async fn remove_subscriber(&self, id: serenity::UserId) -> Result<()> {
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
