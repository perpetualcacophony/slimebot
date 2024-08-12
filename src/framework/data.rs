#[cfg(feature = "wordle")]
use crate::commands::wordle::core::WordleData;

use mongodb::Database;

use chrono::Utc;
use tracing::trace;
use tracing_unwrap::ResultExt;

pub(crate) type UtcDateTime = chrono::DateTime<Utc>;

#[derive(Debug, Clone)]
pub struct PoiseData {
    pub(crate) config: super::config::Config,
    pub(crate) db: Database,
    pub(crate) started: UtcDateTime,

    #[cfg(feature = "wordle")]
    pub(crate) wordle: WordleData,

    //error_tx: ErrorSender,
    minecraft: crate::commands::minecraft::Data,

    #[cfg(feature = "nortverse")]
    nortverse: crate::commands::nortverse::Nortverse,
}

impl PoiseData {
    pub(crate) fn new() -> Self {
        let config: super::config::Config = ::config::Config::builder()
            .add_source(::config::File::with_name("slimebot.toml"))
            .add_source(::config::Environment::with_prefix("SLIMEBOT"))
            .build()
            .expect_or_log("config file could not be loaded")
            .try_deserialize()
            .expect_or_log("configuration could not be parsed");

        trace!("config loaded");

        let db = super::db::database(&config.db);

        let started = Utc::now();

        #[cfg(feature = "wordle")]
        let wordle = WordleData::new(&db);

        /* let (error_tx, error_rx) = ErrorHandler::channel();
        error_rx.spawn(); */

        let minecraft = crate::commands::minecraft::Data::new_mongodb(&db);

        #[cfg(feature = "nortverse")]
        let nortverse = crate::commands::nortverse::Nortverse::from_database(&db);

        Self {
            config,
            db,
            started,

            #[cfg(feature = "wordle")]
            wordle,

            minecraft,

            #[cfg(feature = "nortverse")]
            nortverse,
        }
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
}
