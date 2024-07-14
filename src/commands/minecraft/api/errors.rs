use thisslime::TracingError;

#[derive(Debug, thiserror::Error, TracingError)]
#[span]
pub enum Error {
    #[error(transparent)]
    #[event(level = WARN)]
    ParseUrl(#[from] url::ParseError),

    #[error(transparent)]
    MinecraftApi(#[from] MinecraftApiError),

    #[error(transparent)]
    ImageHost(#[from] ImageHostError),

    #[error(transparent)]
    Client(#[from] ReqwestClientError),
}

#[derive(Debug, thiserror::Error, TracingError)]
#[error("minecraft api returned an error: {source}")]
#[event(level = ERROR)]
pub(crate) struct MinecraftApiError {
    #[field(print = Debug)]
    #[from]
    pub(crate) source: reqwest::Error,
}

#[derive(Debug, thiserror::Error, TracingError)]
#[error("image host returned an error")]
#[event(level = WARN)]
pub(crate) struct ImageHostError {
    #[field(print = Debug)]
    #[from]
    pub(crate) source: reqwest::Error,
}

#[derive(Debug, thiserror::Error, TracingError)]
#[error("error from reqwest client")]
#[event(level = ERROR)]
pub(crate) struct ReqwestClientError {
    #[field(print = Debug)]
    #[from]
    pub(crate) source: reqwest::Error,
}

impl ReqwestClientError {
    pub(crate) fn or_server<Err>(err: reqwest::Error) -> Error
    where
        Err: From<reqwest::Error>,
        Error: From<Err>,
    {
        if err.is_status() {
            Error::from(Err::from(err))
        } else {
            Error::Client(Self::from(err))
        }
    }
}
