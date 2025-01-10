use poise::serenity_prelude as serenity;

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

    pub async fn subscribers(&self) -> Result<impl Iterator<Item = serenity::UserId>> {
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
