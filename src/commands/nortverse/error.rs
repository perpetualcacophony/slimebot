use poise::serenity_prelude as serenity;

#[derive(Debug, thiserror::Error)]
pub enum NortverseError<Data> {
    #[error("data error: {0}")]
    Data(Data),

    #[error(transparent)]
    AlreadySubscribed(#[from] AlreadySubscribedError),

    #[error(transparent)]
    NotSubscribed(#[from] NotSubscribedError),

    #[error("error from reqwest: {0}")]
    Reqwest(#[from] reqwest::Error),
}

impl<Data> NortverseError<Data> {
    pub fn data(error: Data) -> Self {
        Self::Data(error)
    }

    pub fn already_subscribed(user_id: serenity::UserId) -> Self {
        Self::AlreadySubscribed(AlreadySubscribedError { user_id })
    }

    pub fn not_subscribed(user_id: serenity::UserId) -> Self {
        Self::NotSubscribed(NotSubscribedError { user_id })
    }
}

#[derive(Clone, Debug, thiserror::Error)]
#[error("user {user_id} is already subscribed")]
pub struct AlreadySubscribedError {
    user_id: serenity::UserId,
}

#[derive(Clone, Debug, thiserror::Error)]
#[error("user {user_id} is not subscribed")]
pub struct NotSubscribedError {
    user_id: serenity::UserId,
}
