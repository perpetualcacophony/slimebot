#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct DbEnvironment {
    #[serde(default)]
    url: Url,
}

impl DbEnvironment {
    pub fn url(&self) -> &mongodb::options::ServerAddress {
        &self.url.inner
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(from = "mongodb::options::ServerAddress", into = "String")]
struct Url {
    inner: mongodb::options::ServerAddress,
}

impl Default for Url {
    fn default() -> Self {
        mongodb::options::ServerAddress::Tcp {
            host: "db".to_owned(),
            port: Some(27017),
        }
        .into()
    }
}

impl From<mongodb::options::ServerAddress> for Url {
    fn from(value: mongodb::options::ServerAddress) -> Self {
        Self { inner: value }
    }
}

impl From<Url> for String {
    fn from(value: Url) -> Self {
        value.inner.to_string()
    }
}
