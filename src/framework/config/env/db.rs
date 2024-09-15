use mongodb::options::ServerAddress;

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
#[serde(from = "Partial")]
pub struct DbEnvironment {
    url: ServerAddress,
}

impl DbEnvironment {
    pub fn url(&self) -> &ServerAddress {
        &self.url
    }
}

impl From<Partial> for DbEnvironment {
    fn from(partial: Partial) -> Self {
        Self {
            url: partial.url.unwrap_or_else(|| ServerAddress::Tcp {
                host: "db".to_owned(),
                port: Some(27017),
            }),
        }
    }
}

#[derive(serde::Deserialize)]
struct Partial {
    url: Option<ServerAddress>,
}
