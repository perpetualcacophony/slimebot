use vaultrs::client::{VaultClient, VaultClientSettingsBuilder};

pub struct Store<'a> {
    client: VaultClient,
    kv1_mount: &'a str,
}

impl<'a> Store<'a> {
    pub fn from_env(env: &'a super::super::env::Vault) -> Self {
        Self {
            client: VaultClient::new(
                VaultClientSettingsBuilder::default()
                    .address(env.url())
                    .build()
                    .expect("building vault settings should not fail"),
            )
            .expect("building vault client should not fail"),
            kv1_mount: env.kv1_mount(),
        }
    }

    pub async fn load(&self) -> Result<super::Secrets, super::Error> {
        let partial: Secrets = vaultrs::kv1::get(&self.client, self.kv1_mount, "slimebot")
            .await
            .map_err(|err| super::Error::BackendError(Box::new(err)))?;

        Ok(super::Secrets {
            bot_token: partial.bot_token,
            db: Some(super::DbSecrets {
                username: partial.db_username,
                password: partial.db_password,
            }),
        })
    }
}

#[derive(serde::Deserialize)]
struct Secrets {
    bot_token: String,
    db_username: String,
    db_password: String,
}
