#[derive(Debug, thiserror::Error)]
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
    pub vault: Vault<'a>,
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

mod vault;
pub use vault::VaultEnvironment as Vault;
