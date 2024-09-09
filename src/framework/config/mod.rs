mod app;
pub use app::AppConfig as Config;

mod env;
pub use env::EnvConfig as Environment;

mod secrets;
pub use secrets::Secrets;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("problem loading environment: {0}")]
    Env(#[from] env::Error),

    #[error("problem loading config file: {0}")]
    App(#[from] app::Error),

    #[error("problem loading secrets: {0}")]
    Secrets(#[from] secrets::Error),
}

#[derive(Debug, Clone)]
pub struct Configuration {
    app: Config,
    env: Environment,
    secrets: Secrets,
}

impl Configuration {
    pub async fn load() -> Result<Self, Error> {
        let env = Environment::load()?;
        let app = Config::load(&env)?;
        let secrets = Secrets::load().await?;

        Ok(Self { env, app, secrets })
    }

    pub fn mongodb(&self) -> mongodb::options::ClientOptions {
        #[cfg(feature = "vault")]
        let credential = mongodb::options::Credential::builder()
            .username(self.secrets.db_username().to_owned())
            .password(self.secrets.db_password().to_owned())
            .build();

        #[cfg(not(feature = "vault"))]
        let credential = None;

        mongodb::options::ClientOptions::builder()
            .app_name("slimebot".to_owned())
            .credential(credential)
            .hosts(vec![self.env.db_url().clone()])
            .build()
    }

    pub fn token(&self) -> &str {
        self.secrets.bot_token()
    }
}

mod as_ref {
    use super::*;

    impl AsRef<Config> for Configuration {
        fn as_ref(&self) -> &Config {
            &self.app
        }
    }

    impl AsRef<Environment> for Configuration {
        fn as_ref(&self) -> &Environment {
            &self.env
        }
    }

    impl AsRef<Secrets> for Configuration {
        fn as_ref(&self) -> &Secrets {
            &self.secrets
        }
    }
}
