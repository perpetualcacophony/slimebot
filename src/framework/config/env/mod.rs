#[derive(Debug, thiserror::Error, thisslime::TracingError)]
#[error("couldn't load environment variable '{key}': {message}")]
pub struct Error {
    key: &'static str,
    message: &'static str,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct Environment<'a> {
    token: Option<&'a str>,
    config_file: &'a str,
    pub db: Db,
    pub secrets: Secrets<'a>,
}

impl<'a> Environment<'a> {
    pub fn token(&self) -> Option<&str> {
        self.token
    }

    pub fn config_file(&self) -> &'a str {
        self.config_file
    }

    pub fn load() -> Result<Self, Error> {
        todo!()
    }
}

mod db;
pub use db::DbEnvironment as Db;

#[cfg(feature = "vault")]
mod vault;

#[cfg(feature = "vault")]
pub use vault::VaultEnvironment as Vault;

#[derive(serde::Deserialize, Debug, Clone)]
pub enum Secrets<'a> {
    Dev {
        token: &'a str,
    },

    #[cfg(feature = "vault")]
    Vault(Vault<'a>),
}
