use url::Url;

const URL_DEFAULT: fn() -> Url =
    || Url::parse("http://vault:8200").expect("hard-coded url should be valid");

const KV1_MOUNT_DEFAULT: fn() -> String = || "slimebot".to_owned();

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct VaultEnvironment {
    #[serde(default = "URL_DEFAULT")]
    url: Url,

    #[serde(default = "KV1_MOUNT_DEFAULT")]
    kv1_mount: String,
}

impl VaultEnvironment {
    pub fn url(&self) -> &Url {
        &self.url
    }

    pub fn kv1_mount(&self) -> &str {
        &self.kv1_mount
    }
}
