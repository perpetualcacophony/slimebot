use poise::serenity_prelude as serenity;

mod mongodb;

pub trait NortverseData {
    type Error;

    fn latest_slug(&self) -> Result<Option<String>, Self::Error>;
    fn set_latest(&mut self, slug: String) -> Result<(), Self::Error>;

    fn subscribers(&self) -> Result<impl IntoIterator<Item = serenity::UserId>, Self::Error>;
    fn add_subscriber(&mut self, id: serenity::UserId) -> Result<(), Self::Error>;
    fn remove_subscriber(&mut self, id: serenity::UserId) -> Result<(), Self::Error>;

    fn get_subscriber(
        &self,
        id: &serenity::UserId,
    ) -> Result<Option<serenity::UserId>, Self::Error> {
        Ok(self.subscribers()?.into_iter().find(|item| item == id))
    }
}

pub trait NortverseDataAsync {
    type Error;

    async fn latest_slug(&self) -> Result<Option<String>, Self::Error>;
    async fn set_latest(&mut self, slug: String) -> Result<(), Self::Error>;

    async fn subscribers(&self) -> Result<impl IntoIterator<Item = serenity::UserId>, Self::Error>;
    async fn add_subscriber(&mut self, id: serenity::UserId) -> Result<(), Self::Error>;
    async fn remove_subscriber(&mut self, id: serenity::UserId) -> Result<(), Self::Error>;

    async fn get_subscriber(
        &self,
        id: &serenity::UserId,
    ) -> Result<Option<serenity::UserId>, Self::Error> {
        Ok(self
            .subscribers()
            .await?
            .into_iter()
            .find(|item| item == id))
    }
}

impl<T: NortverseData> NortverseDataAsync for T {
    type Error = T::Error;

    async fn latest_slug(&self) -> Result<Option<String>, Self::Error> {
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
