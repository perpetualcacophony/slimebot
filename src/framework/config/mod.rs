mod app;
use std::ops::Deref;

pub use app::AppConfig as Config;

pub mod env;
pub use env::Environment;

mod secrets;
pub use secrets::Secrets;

#[derive(Debug, thiserror::Error, thisslime::TracingError)]
pub enum Error {
    #[error("problem loading environment: {0}")]
    Env(#[from] env::Error),

    #[error("problem loading config file: {0}")]
    App(#[from] app::Error),

    #[error("problem loading secrets: {0}")]
    Secrets(#[from] secrets::Error),
}

#[derive(Debug, Clone)]
pub struct ConfigSetup {
    app: Config,
    pub env: Environment,
    secrets: Secrets,
}

impl ConfigSetup {
    #[tracing::instrument(skip_all, name = "config")]
    pub async fn load(#[cfg(feature = "cli")] cli: &crate::Cli) -> Result<Self, Error> {
        #[cfg(not(feature = "cli"))]
        let env = Environment::load(None)?;

        #[cfg(feature = "cli")]
        let env = Environment::load(cli.env_path.as_ref())?;

        let app = Config::load(&env)?;
        let secrets = Secrets::load(&env).await?;

        Ok(Self { env, app, secrets })
    }

    pub fn mongodb(&self) -> mongodb::options::ClientOptions {
        let credential = self
            .secrets
            .db
            .as_ref()
            .map(secrets::DbSecrets::mongo_credential);

        mongodb::options::ClientOptions::builder()
            .app_name("slimebot".to_owned())
            .credential(credential)
            .hosts(vec![self.env.db.url().clone()])
            .build()
    }

    pub fn token(&self) -> &str {
        self.secrets.bot_token()
    }

    pub fn finish(self) -> Config {
        self.app
    }
}

impl Deref for ConfigSetup {
    type Target = Config;

    fn deref(&self) -> &Self::Target {
        &self.app
    }
}
