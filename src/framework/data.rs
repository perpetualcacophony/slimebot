#[cfg(feature = "wordle")]
use crate::commands::wordle::core::WordleData;

use mongodb::Database;

use chrono::Utc;
use tracing::{info, warn};
use tracing_unwrap::ResultExt;

use super::Secrets;

pub mod error;
pub use error::Error as DataError;

pub type Result<T, E = DataError> = std::result::Result<T, E>;

pub(crate) type UtcDateTime = chrono::DateTime<Utc>;

#[derive(Debug, Clone)]
pub struct PoiseData {
    pub(crate) config: super::config::Config,
    pub(crate) db: Database,
    pub(crate) started: UtcDateTime,

    pub(crate) secrets: Secrets,

    #[cfg(feature = "wordle")]
    pub(crate) wordle: WordleData,

    //error_tx: ErrorSender,
    minecraft: crate::commands::minecraft::Data,

    #[cfg(feature = "nortverse")]
    nortverse: crate::commands::nortverse::Nortverse,

    #[cfg(feature = "dynasty")]
    dynasty: dynasty2::Dynasty,
}

impl PoiseData {
    pub(crate) async fn new() -> Result<Self> {
        dotenvy::dotenv().ok();

        let nvee_path = if cfg!(feature = "docker") {
            "/slimebot.nvee"
        } else {
            "slimebot.nvee"
        };

        nvee::from_path(nvee_path).ok();

        let config_file = if let Ok(path) = std::env::var("SLIMEBOT_TOML") {
            info!(path, "looking for config file with SLIMEBOT_TOML...");
            path
        } else {
            #[cfg(not(feature = "docker"))]
            let path = "./slimebot.toml".to_owned();

            #[cfg(feature = "docker")]
            let path = "/slimebot.toml".to_owned();

            warn!(path, "SLIMEBOT_TOML env unset, using default path");
            path
        };

        let config: super::config::Config = ::config::Config::builder()
            .add_source(::config::File::new(&config_file, config::FileFormat::Toml))
            .build()
            .expect_or_log("config file could not be loaded")
            .try_deserialize()
            .expect_or_log("configuration could not be parsed");

        info!("config loaded");

        #[cfg(feature = "vault")]
        let secrets = Secrets::from_vault().await?;

        #[cfg(not(feature = "vault"))]
        let secrets = Secrets::secret_files(&config.secrets_dir())
            .await
            .map_err(crate::framework::secrets::Error::from)?;

        let db = super::db::database(&secrets);

        let started = Utc::now();

        #[cfg(feature = "wordle")]
        let wordle = WordleData::new(&db);

        /* let (error_tx, error_rx) = ErrorHandler::channel();
        error_rx.spawn(); */

        let minecraft = crate::commands::minecraft::Data::new_mongodb(&db);

        #[cfg(feature = "nortverse")]
        let nortverse = crate::commands::nortverse::Nortverse::from_database(&db);

        #[cfg(feature = "dynasty")]
        let dynasty = dynasty2::Dynasty::new();

        Ok(Self {
            config,
            db,
            started,

            secrets,

            #[cfg(feature = "wordle")]
            wordle,

            minecraft,

            #[cfg(feature = "nortverse")]
            nortverse,

            #[cfg(feature = "dynasty")]
            dynasty,
        })
    }

    pub(crate) const fn config(&self) -> &super::config::Config {
        &self.config
    }

    #[allow(dead_code)]
    pub(crate) const fn db(&self) -> &Database {
        &self.db
    }

    #[cfg(feature = "wordle")]
    pub(crate) const fn wordle(&self) -> &WordleData {
        &self.wordle
    }

    /* pub(crate) fn error_tx(&self) -> ErrorSender {
        self.error_tx.clone()
    } */

    pub(crate) const fn minecraft(&self) -> &crate::commands::minecraft::Data {
        &self.minecraft
    }

    #[cfg(feature = "nortverse")]
    pub(crate) const fn nortverse(&self) -> &crate::commands::nortverse::Nortverse {
        &self.nortverse
    }

    #[cfg(feature = "dynasty")]
    pub(crate) const fn dynasty(&self) -> &dynasty2::Dynasty {
        &self.dynasty
    }
}
