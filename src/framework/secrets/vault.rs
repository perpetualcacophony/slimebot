use url::Url;
use vaultrs::client::{VaultClient, VaultClientSettingsBuilder};

pub async fn secrets() -> Result<super::Secrets, super::Error> {
    let env = Environment::get().map_err(|err| super::Error::BackendError(Box::new(err)))?;

    let vault = VaultClient::new(
        VaultClientSettingsBuilder::default()
            .address(&env.url)
            .token(&env.token)
            .build()
            .expect("building vault settings should not fail"),
    )
    .expect("building vault client should not fail");

    vaultrs::kv1::get(&vault, env.kv1_mount(), "slimebot")
        .await
        .map_err(|err| super::Error::BackendError(Box::new(err)))
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Environment {
    // SLIMEBOT_VAULT_URL
    url: Url,

    // SLIMEBOT_VAULT_KV1_MOUNT
    kv1_mount: String,

    // SLIMEBOT_VAULT_TOKEN
    token: String,
}

impl Environment {
    const SLIMEBOT_VAULT_URL: &str = "SLIMEBOT_VAULT_URL";
    const SLIMEBOT_VAULT_KV1_MOUNT: &str = "SLIMEBOT_VAULT_KV1_MOUNT";
    const SLIMEBOT_VAULT_TOKEN: &str = "SLIMEBOT_VAULT_TOKEN";

    pub fn get() -> Result<Self, EnvVarError> {
        use std::env;

        Ok(Self {
            url: Url::parse(
                &env::var(Self::SLIMEBOT_VAULT_URL).map_err(|_| EnvVarError {
                    var: Self::SLIMEBOT_VAULT_URL,
                    meta: EnvVarErrorMeta::NotFound,
                })?,
            )
            .map_err(|inner| EnvVarError {
                var: Self::SLIMEBOT_VAULT_URL,
                meta: EnvVarErrorMeta::Invalid(Box::new(inner)),
            })?,
            kv1_mount: env::var(Self::SLIMEBOT_VAULT_KV1_MOUNT).map_err(|_| EnvVarError {
                var: Self::SLIMEBOT_VAULT_KV1_MOUNT,
                meta: EnvVarErrorMeta::NotFound,
            })?,
            token: env::var(Self::SLIMEBOT_VAULT_TOKEN).map_err(|_| EnvVarError {
                var: Self::SLIMEBOT_VAULT_TOKEN,
                meta: EnvVarErrorMeta::NotFound,
            })?,
        })
    }

    pub fn kv1_mount(&self) -> &str {
        self.kv1_mount.as_str()
    }
}

#[derive(Debug, thiserror::Error)]
#[error("error loading env var '{var}' ({meta:?})")]
pub struct EnvVarError {
    var: &'static str,
    meta: EnvVarErrorMeta,
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum EnvVarErrorMeta {
    NotFound,
    Invalid(Box<dyn std::error::Error + Send + Sync>),
}
