#[derive(Debug, thiserror::Error, thisslime::TracingError)]
pub enum Error {
    #[error("io error: {0}")]
    #[event(level = ERROR)]
    Io(#[from] std::io::Error),

    #[error("couldn't read toml: {0}")]
    #[event(level = ERROR)]
    Toml(#[from] toml::de::Error),
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct Environment {
    config_file: String,
    pub db: Db,
    pub secrets: Secrets,
}

impl Environment {
    pub fn config_file(&self) -> &str {
        &self.config_file
    }

    #[tracing::instrument(skip_all, name = "env")]
    pub fn load(path: Option<&Path>) -> Result<Self, Error> {
        use serde::Deserialize;

        let path = path
            .cloned()
            .unwrap_or_else(|| Path::from_var().unwrap_or_default());
        tracing::debug!(?path, "looking for environment configuration at {path:?}");

        let text = path.read()?;
        let deserializer = toml::Deserializer::new(&text);
        let result = Self::deserialize(deserializer)?;

        tracing::debug!("done!");

        Ok(result)
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
#[serde(transparent)]
pub struct Path {
    inner: std::path::PathBuf,
}

impl Path {
    fn new(s: &str) -> Self {
        Self {
            inner: std::path::Path::new(s).to_path_buf(),
        }
    }

    fn from_string(s: String) -> Self {
        Self { inner: s.into() }
    }

    fn from_var() -> Option<Self> {
        std::env::var("SLIMEBOT_ENV_PATH")
            .map(Self::from_string)
            .ok()
    }

    fn read(&self) -> std::io::Result<String> {
        std::fs::read_to_string(&self.inner)
    }
}

impl Default for Path {
    fn default() -> Self {
        Self::new("/slimebot/config/env.toml")
    }
}

impl std::fmt::Debug for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::path::Path::fmt(&self.inner, f)
    }
}

impl std::str::FromStr for Path {
    type Err = <std::path::PathBuf as std::str::FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            inner: std::str::FromStr::from_str(s)?,
        })
    }
}

mod db;
pub use db::DbEnvironment as Db;

#[cfg(feature = "vault")]
mod vault;

#[cfg(feature = "vault")]
pub use vault::VaultEnvironment as Vault;

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum Secrets {
    Dev {
        token: String,
    },

    #[cfg(feature = "vault")]
    Vault(Vault),
}
