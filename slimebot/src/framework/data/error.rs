use crate::framework::config;

#[derive(Debug, thiserror::Error, thisslime::TracingError)]
pub enum Error {
    #[error("error loading configuration: {0}")]
    #[event(level = ERROR)]
    Config(#[from] config::Error),
}
