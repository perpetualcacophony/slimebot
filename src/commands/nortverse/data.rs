use poise::serenity_prelude as serenity;

use super::Error;

mod mongodb;
pub use mongodb::MongoDb;

pub trait NortverseData {
    type Error: std::error::Error;

    fn latest_slug(&self) -> Result<Option<impl AsRef<str>>, Self::Error>;
    fn set_latest(&mut self, slug: String) -> Result<(), Self::Error>;

    fn subscribers(&self) -> Result<impl IntoIterator<Item = serenity::UserId>, Self::Error>;
    fn add_subscriber(&mut self, id: serenity::UserId) -> Result<(), Self::Error>;
    fn remove_subscriber(&mut self, id: serenity::UserId) -> Result<(), Self::Error>;

    fn contains_subscriber(&self, id: &serenity::UserId) -> Result<bool, Self::Error> {
        Ok(self.subscribers()?.into_iter().any(|item| &item == id))
    }
}

pub trait NortverseDataAsync {
    type Error: std::error::Error;

    async fn latest_slug(&self) -> Result<Option<impl AsRef<str>>, Self::Error>;
    async fn set_latest(&mut self, slug: String) -> Result<(), Self::Error>;

    async fn subscribers(&self) -> Result<impl IntoIterator<Item = serenity::UserId>, Self::Error>;
    async fn add_subscriber(&mut self, id: serenity::UserId) -> Result<(), Self::Error>;
    async fn remove_subscriber(&mut self, id: serenity::UserId) -> Result<(), Self::Error>;

    async fn contains_subscriber(&self, id: &serenity::UserId) -> Result<bool, Self::Error> {
        Ok(self
            .subscribers()
            .await?
            .into_iter()
            .any(|item| &item == id))
    }
}

impl<T: NortverseData> NortverseDataAsync for T {
    type Error = T::Error;

    async fn latest_slug(&self) -> Result<Option<impl AsRef<str>>, Self::Error> {
        NortverseData::latest_slug(self)
    }

    async fn set_latest(&mut self, slug: String) -> Result<(), Self::Error> {
        NortverseData::set_latest(self, slug)
    }

    async fn subscribers(&self) -> Result<impl IntoIterator<Item = serenity::UserId>, Self::Error> {
        NortverseData::subscribers(self)
    }

    async fn add_subscriber(&mut self, id: serenity::UserId) -> Result<(), Self::Error> {
        NortverseData::add_subscriber(self, id)
    }

    async fn remove_subscriber(&mut self, id: serenity::UserId) -> Result<(), Self::Error> {
        NortverseData::remove_subscriber(self, id)
    }
}

#[derive(Clone, Debug)]
pub struct Std {
    latest: String,
    subscribers: std::collections::HashSet<serenity::UserId>,
}

impl NortverseData for Std {
    type Error = std::convert::Infallible;

    fn latest_slug(&self) -> Result<Option<impl AsRef<str>>, Self::Error> {
        Ok(Some(&self.latest))
    }

    fn set_latest(&mut self, slug: String) -> Result<(), Self::Error> {
        self.latest = slug;
        Ok(())
    }

    fn subscribers(&self) -> Result<impl IntoIterator<Item = serenity::UserId>, Self::Error> {
        Ok(self.subscribers.iter().copied())
    }

    fn add_subscriber(&mut self, id: serenity::UserId) -> Result<(), Self::Error> {
        self.subscribers.insert(id);
        Ok(())
    }

    fn remove_subscriber(&mut self, id: serenity::UserId) -> Result<(), Self::Error> {
        self.subscribers.remove(&id);
        Ok(())
    }

    fn contains_subscriber(&self, id: &serenity::UserId) -> Result<bool, Self::Error> {
        Ok(self.subscribers.contains(id))
    }
}
