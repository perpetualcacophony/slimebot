use url::Url;
use vaultrs::{
    client::{VaultClient, VaultClientSettingsBuilder},
    kv1,
};

use super::SecretStore;

pub struct VaultSecrets {
    vault: VaultClient,
    env: Environment,
}

impl VaultSecrets {
    pub fn new() -> Self {
        let env = Environment::get().unwrap();

        let vault =
            VaultClient::new(VaultClientSettingsBuilder::default().build().unwrap()).unwrap();

        Self { vault, env }
    }
}

impl SecretStore for VaultSecrets {
    async fn try_get(&self, secret: super::SecretKey) -> Option<String> {
        kv1::get(&self.vault, self.env.kv1_mount(), &secret.to_string())
            .await
            .ok()
    }
}

struct Environment {
    // SLIMEBOT_VAULT_URL
    url: Url,

    // SLIMEBOT_VAULT_KV1_MOUNT
    kv1_mount: String,
}

impl Environment {
    const SLIMEBOT_VAULT_URL: &str = "SLIMEBOT_VAULT_URL";
    const SLIMEBOT_VAULT_KV1_MOUNT: &str = "SLIMEBOT_VAULT_KV1_MOUNT";

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
        })
    }

    pub fn kv1_mount(&self) -> &str {
        self.kv1_mount.as_str()
    }
}

#[derive(Debug)]
pub struct EnvVarError {
    var: &'static str,
    meta: EnvVarErrorMeta,
}

#[derive(Debug)]
pub enum EnvVarErrorMeta {
    NotFound,
    Invalid(Box<dyn std::error::Error>),
}
