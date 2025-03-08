mod vault;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Secrets {
    bot_token: String,
    pub db: Option<DbSecrets>,
}

impl Secrets {
    pub fn bot_token(&self) -> &str {
        &self.bot_token
    }

    #[tracing::instrument(skip_all, name = "secrets")]
    pub async fn load(env: &super::Environment) -> Result<Self, Error> {
        match &env.secrets {
            super::env::Secrets::Dev { token_file } => {
                tracing::warn!("loading token from file at {token_file}");
                Ok(Self {
                    bot_token: std::fs::read_to_string(token_file)
                        .map_err(|err| Error::BackendError(Box::new(err)))?,
                    db: None,
                })
            }
            super::env::Secrets::Vault(vault) => vault::Store::from_env(vault).load().await,
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct DbSecrets {
    username: String,
    password: String,
}

impl DbSecrets {
    pub fn mongo_credential(&self) -> mongodb::options::Credential {
        mongodb::options::Credential::builder()
            .username(self.username.clone())
            .password(self.password.clone())
            .build()
    }
}

#[allow(dead_code)]
#[derive(Debug, thiserror::Error, thisslime::TracingError)]
pub enum Error {
    #[error("error when fetching secrets: {0:?}")]
    #[event(level = ERROR)]
    BackendError(Box<dyn std::error::Error + Send + Sync>),
}
