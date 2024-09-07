use crate::framework::secrets;

#[derive(Debug, thiserror::Error, thisslime::TracingError)]
pub enum Error {
    #[error("error loading secrets: {0}")]
    #[event(level = ERROR)]
    Secrets(#[from] secrets::Error),
}
