use url::Url;

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(from = "Partial")]
pub struct VaultEnvironment<'a> {
    url: Url,
    kv1_mount: &'a str,
}

impl VaultEnvironment<'_> {
    const URL_DEFAULT: fn() -> Url =
        || Url::parse("http://vault:8200").expect("hard-coded url should be valid");

    const KV1_MOUNT_DEFAULT: &'static str = "slimebot";
}

impl<'a> VaultEnvironment<'a> {
    pub fn url(&self) -> &Url {
        &self.url
    }

    pub fn kv1_mount(&self) -> &'a str {
        self.kv1_mount
    }
}

impl<'a> From<Partial<'a>> for VaultEnvironment<'a> {
    fn from(partial: Partial<'a>) -> Self {
        Self {
            url: partial.url.unwrap_or_else(Self::URL_DEFAULT),
            kv1_mount: partial.kv1_mount.unwrap_or(Self::KV1_MOUNT_DEFAULT),
        }
    }
}

#[derive(serde::Deserialize)]
struct Partial<'a> {
    url: Option<Url>,
    kv1_mount: Option<&'a str>,
}
