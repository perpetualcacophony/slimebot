use poise::serenity_prelude as serenity;
use serenity::futures;

pub trait NortverseData {
    type Error;

    fn latest_slug(&self) -> Result<String, Self::Error>;
    fn set_latest(&mut self, slug: &str) -> Result<(), Self::Error>;

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
