#[cfg(feature = "wordle")]
use crate::commands::wordle::core::WordleData;

use mongodb::Database;

use chrono::Utc;

use super::{config::ConfigSetup, Config};

pub mod error;
pub use error::Error as DataError;

pub type Result<T, E = DataError> = std::result::Result<T, E>;

pub(crate) type UtcDateTime = chrono::DateTime<Utc>;

#[derive(Debug, Clone)]
pub struct PoiseData {
    config: Config,
    pub(crate) db: Database,
    pub(crate) started: UtcDateTime,

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
    pub(crate) async fn new(config: ConfigSetup<'_>) -> Result<Self> {
        dotenvy::dotenv().ok();

        let nvee_path = if cfg!(feature = "docker") {
            "/slimebot.nvee"
        } else {
            "slimebot.nvee"
        };

        nvee::from_path(nvee_path).ok();

        let db = mongodb::Client::with_options(config.mongodb())
            .expect("building client should not fail")
            .database("slimebot");

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
            config: config.finish(),
            db,
            started,

            #[cfg(feature = "wordle")]
            wordle,

            minecraft,

            #[cfg(feature = "nortverse")]
            nortverse,

            #[cfg(feature = "dynasty")]
            dynasty,
        })
    }

    pub(crate) fn config(&self) -> &Config {
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
