use crate::framework::secrets::MissingSecretError;

#[derive(Debug, thiserror::Error, thisslime::TracingError)]
pub enum Error {
    #[error(transparent)]
    #[event(level = ERROR)]
    MissingSecret(#[from] MissingSecretError),
}
