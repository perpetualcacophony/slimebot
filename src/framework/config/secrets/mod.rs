use std::{fmt::Display, path::Path};

#[cfg(feature = "vault")]
mod vault;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Secrets {
    bot_token: String,

    #[cfg(feature = "db_auth")]
    db_username: String,

    #[cfg(feature = "db_auth")]
    db_password: String,
}

impl Secrets {
    #[cfg(feature = "db_auth")]
    pub async fn from_store(store: impl SecretStore) -> Result<Self, MissingSecretError> {
        let (bot_token, db_username, db_password) = tokio::try_join!(
            store.get(SecretKey::BotToken),
            store.get(SecretKey::DbUsername),
            store.get(SecretKey::DbPassword)
        )?;

        Ok(Self {
            bot_token,
            db_username,
            db_password,
        })
    }

    #[cfg(feature = "db_auth")]
    pub async fn secret_files(dir: &Path) -> Result<Self, MissingSecretError> {
        Self::from_store(SecretFiles { directory: dir }).await
    }

    #[cfg(feature = "vault")]
    pub async fn from_vault() -> Result<Self, Error> {
        vault::secrets().await
    }

    #[cfg(not(feature = "db_auth"))]
    pub fn dev() -> Result<Self, Error> {
        Ok(Self {
            bot_token: std::env::var("SLIMEBOT_TOKEN").map_err(|_| MissingSecretError {
                secret: SecretKey::BotToken,
            })?,
        })
    }

    pub async fn load() -> Result<Self, Error> {
        todo!()
    }
}

impl Secrets {
    pub fn bot_token(&self) -> &str {
        &self.bot_token
    }

    #[cfg(feature = "db_auth")]
    pub fn db_username(&self) -> &str {
        &self.db_username
    }

    #[cfg(feature = "db_auth")]
    pub fn db_password(&self) -> &str {
        &self.db_password
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SecretKey {
    BotToken,

    #[cfg(feature = "db_auth")]
    DbUsername,

    #[cfg(feature = "db_auth")]
    DbPassword,
}

impl Display for SecretKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::BotToken => "bot_token",

            #[cfg(feature = "db_auth")]
            Self::DbPassword => "db_password",

            #[cfg(feature = "db_auth")]
            Self::DbUsername => "db_username",
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    MissingSecret(#[from] MissingSecretError),

    #[error("error when fetching secrets: {0:?}")]
    BackendError(Box<dyn std::error::Error + Send + Sync>),
}

#[derive(Debug, thiserror::Error)]
#[error("missing secret `{secret}`")]
pub struct MissingSecretError {
    secret: SecretKey,
}

pub trait SecretStore {
    async fn try_get(&self, secret: SecretKey) -> Option<String>;

    async fn get(&self, secret: SecretKey) -> Result<String, MissingSecretError> {
        if let Some(value) = self.try_get(secret).await {
            tracing::trace!("loaded secret `{secret}`");
            Ok(value)
        } else {
            Err(MissingSecretError { secret })
        }
    }
}

pub struct SecretFiles<'path> {
    directory: &'path Path,
}

impl SecretStore for SecretFiles<'_> {
    async fn try_get(&self, secret: SecretKey) -> Option<String> {
        tokio::fs::read_to_string(self.directory.join(secret.to_string()))
            .await
            .ok()
            .map(|s| {
                s.lines()
                    .next()
                    .expect("secret file should not be empty")
                    .to_owned()
            })
    }
}
