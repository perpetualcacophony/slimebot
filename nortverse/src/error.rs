use poise::serenity_prelude as serenity;

#[derive(Debug, thiserror::Error, thisslime::TracingError)]
pub enum NortverseError {
    #[error("data error: {0}")]
    #[event]
    Data(String),

    #[error(transparent)]
    ParseComic(#[from] super::comic::ParseError),

    #[error(transparent)]
    AlreadySubscribed(#[from] AlreadySubscribedError),

    #[error(transparent)]
    NotSubscribed(#[from] NotSubscribedError),

    #[error("error from reqwest: {0}")]
    #[event]
    Reqwest(#[from] reqwest::Error),
}

impl NortverseError {
    pub fn data(error: impl std::error::Error) -> Self {
        Self::Data(error.to_string())
    }

    pub fn already_subscribed(user_id: serenity::UserId) -> Self {
        Self::AlreadySubscribed(AlreadySubscribedError { user_id })
    }

    pub fn not_subscribed(user_id: serenity::UserId) -> Self {
        Self::NotSubscribed(NotSubscribedError { user_id })
    }
}

#[derive(Clone, Debug, thiserror::Error, thisslime::TracingError)]
#[error("user {user_id} is already subscribed")]
#[event(level = WARN)]
pub struct AlreadySubscribedError {
    #[field(print = Display)]
    user_id: serenity::UserId,
}

#[derive(Clone, Debug, thiserror::Error, thisslime::TracingError)]
#[error("user {user_id} is not subscribed")]
#[event(level = WARN)]
pub struct NotSubscribedError {
    #[field(print = Display)]
    user_id: serenity::UserId,
}
