use std::path::PathBuf;

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(try_from = "Partial")]
pub struct EnvConfig {
    db_url: mongodb::options::ServerAddress,

    config_file: String,
}

impl EnvConfig {
    pub(super) fn load() -> Result<Self, Error> {
        todo!()
    }

    fn from_partial(partial: Partial) -> Result<Self, Error> {
        Ok(Self {
            db_url: partial.db_url,
            config_file: partial
                .config_file
                .unwrap_or_else(|| PathBuf::from("./slimebot.toml"))
                .to_str()
                .ok_or(Error {
                    key: "SLIMEBOT_CONFIG_FILE",
                    message: "path to configuration must be valid UTF-8",
                })?
                .to_owned(),
        })
    }

    pub fn db_url(&self) -> &mongodb::options::ServerAddress {
        &self.db_url
    }

    pub fn config_file(&self) -> &str {
        &self.config_file
    }
}

impl TryFrom<Partial> for EnvConfig {
    type Error = Error;

    fn try_from(value: Partial) -> Result<Self, Self::Error> {
        Self::from_partial(value)
    }
}

#[derive(serde::Deserialize)]
struct Partial {
    db_url: mongodb::options::ServerAddress,

    config_file: Option<PathBuf>,
}

#[derive(Debug, thiserror::Error)]
#[error("couldn't load environment variable '{key}': {message}")]
pub struct Error {
    key: &'static str,
    message: &'static str,
}
