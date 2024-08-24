use std::{fmt::Display, path::Path};

pub struct Secrets {
    bot_token: String,
    db_username: String,
    db_password: String,
}

impl Secrets {
    pub async fn from_store(store: impl SecretStore) -> Result<Self, MissingSecretError> {
        let (bot_token, db_username, db_password) = tokio::try_join!(
            store.get2(SecretKey::BotToken),
            store.get2(SecretKey::DbUsername),
            store.get2(SecretKey::DbPassword)
        )
        .map_err(|key: SecretKey| MissingSecretError { secret: key })?;

        Ok(Self {
            bot_token,
            db_username,
            db_password,
        })
    }

    pub async fn secret_files(dir: &Path) -> Result<Self, MissingSecretError> {
        Self::from_store(SecretFiles { directory: dir }).await
    }
}

impl Secrets {
    pub fn bot_token(&self) -> &str {
        &self.bot_token
    }

    pub fn db_username(&self) -> &str {
        &self.db_username
    }

    pub fn db_password(&self) -> &str {
        &self.db_password
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SecretKey {
    BotToken,
    DbUsername,
    DbPassword,
}

impl Display for SecretKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::BotToken => "bot_token",
            Self::DbPassword => "db_password",
            Self::DbUsername => "db_username",
        })
    }
}

pub struct MissingSecretError {
    secret: SecretKey,
}

pub trait SecretStore {
    async fn get(&self, secret: SecretKey) -> Option<String>;

    async fn get2(&self, secret: SecretKey) -> Result<String, SecretKey> {
        self.get(secret).await.ok_or(secret)
    }
}

pub struct SecretFiles<'path> {
    directory: &'path Path,
}

impl SecretStore for SecretFiles<'_> {
    async fn get(&self, secret: SecretKey) -> Option<String> {
        tokio::fs::read_to_string(self.directory.join(secret.to_string()))
            .await
            .ok()
    }
}
