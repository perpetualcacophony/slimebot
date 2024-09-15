use url::Url;

const URL_DEFAULT: fn() -> Url =
    || Url::parse("http://vault:8200").expect("hard-coded url should be valid");

const KV1_MOUNT_DEFAULT: fn() -> &'static str = || "slimebot";

#[derive(serde::Deserialize, Debug, Clone)]
pub struct VaultEnvironment<'a> {
    #[serde(default = "URL_DEFAULT")]
    url: Url,

    #[serde(default = "KV1_MOUNT_DEFAULT")]
    kv1_mount: &'a str,
}

impl<'a> VaultEnvironment<'a> {
    pub fn url(&self) -> &Url {
        &self.url
    }

    pub fn kv1_mount(&self) -> &'a str {
        self.kv1_mount
    }
}
