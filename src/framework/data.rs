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
    pub(crate) wordle: WordleData,
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

        let wordle = WordleData::new(&db);

        Self {
            config,
            db,
            started,
            wordle,
        }
    }

    pub(crate) const fn config(&self) -> &super::config::Config {
        &self.config
    }

    #[allow(dead_code)]
    pub(crate) const fn db(&self) -> &Database {
        &self.db
    }

    pub(crate) const fn wordle(&self) -> &WordleData {
        &self.wordle
    }
}
