#[derive(Debug, thiserror::Error, thisslime::TracingError)]
pub enum ParseComicError {
    #[error(transparent)]
    #[event]
    ParseUrl(#[from] super::url::ParseError),

    #[error(transparent)]
    NoText(#[from] NoTextError),

    #[error(transparent)]
    #[event]
    Reqwest(#[from] reqwest::Error),
}

#[derive(Debug, thiserror::Error, thisslime::TracingError)]
#[error("url {url} missing html body")]
pub struct NoTextError {
    #[field(print = Display)]
    url: reqwest::Url,
}

impl NoTextError {
    pub fn new(url: reqwest::Url) -> Self {
        Self { url }
    }
}
