use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};

use super::Environment;
use poise::serenity_prelude::{ChannelId, RoleId};
use rand::seq::IteratorRandom;
use serde::Deserialize;
use tracing::{error, warn};

mod bot;
pub use bot::BotConfig;

#[derive(Deserialize, Debug, Clone)]
pub struct AppConfig {
    pub secrets_dir: Option<PathBuf>,

    pub bot: BotConfig,
    pub logs: LogsConfig,
    #[serde(default)]
    pub watchers: WatchersConfig,
    #[serde(default)]
    pub bug_reports: BugReportsConfig,

    #[cfg(feature = "wordle")]
    #[serde(default)]
    pub wordle: WordleConfig,
}

impl AppConfig {
    pub async fn setup<'a>() -> Result<super::ConfigSetup<'a>, super::Error> {
        super::ConfigSetup::load().await
    }

    pub(super) fn load(env: &Environment) -> Result<Self, Error> {
        ::config::Config::builder()
            .add_source(::config::File::new(
                env.config_file(),
                config::FileFormat::Toml,
            ))
            .build()
            .map_err(Error::Read)?
            .try_deserialize()
            .map_err(Error::Parse)
    }

    pub fn bug_reports_channel(&self) -> Option<&ChannelId> {
        if let Some(channel) = self.bug_reports.channel() {
            Some(channel)
        } else {
            warn!("bug reports not configured");
            None
        }
    }

    #[tracing::instrument(skip_all)]
    pub fn secrets_dir(&self) -> Cow<Path> {
        tracing::trace!("looking for secrets directory...");

        if let Ok(env) = std::env::var("SLIMEBOT_SECRETS_DIR") {
            tracing::trace!(
                var = "SLIMEBOT_SECRETS_DIR",
                value = env,
                "using value from environment"
            );

            if cfg!(feature = "docker") && env.as_str() != "/etc/slimebot/secrets" {
                tracing::warn!("running in docker, but not using the expected secrets directory. are you sure whatever you're doing is worth it?");
            }

            PathBuf::from(env).into()
        } else if let Some(ref config) = self.secrets_dir {
            tracing::trace!(value = ?config, "using value from config");

            if cfg!(feature = "docker") && config != Path::new("/etc/slimebot/secrets") {
                tracing::warn!("running in docker, but not using the expected secrets directory. are you sure whatever you're doing is worth it?");
            }

            config.into()
        } else if cfg!(feature = "docker") {
            tracing::trace!(
                value = "/etc/slimebot/secrets",
                "using docker default value"
            );

            Path::new("/etc/slimebot/secrets").into()
        } else {
            tracing::error!("no secrets directory specified in config or environment");

            panic!()
        }
    }
}

#[derive(Debug, thiserror::Error, thisslime::TracingError)]
pub enum Error {
    #[error("file read error: {0}")]
    #[event(level = ERROR)]
    Read(config::ConfigError),

    #[error("parsing error: {0}")]
    #[event(level = ERROR)]
    Parse(config::ConfigError),
}

#[derive(Deserialize, Debug, Clone)]
pub struct LogsConfig {
    flavor_texts: Vec<String>,
}

impl LogsConfig {
    pub fn flavor_text(&self) -> Option<&str> {
        let flavor_text = self
            .flavor_texts
            .iter()
            .choose(&mut rand::thread_rng())
            .map(|s| s.as_str());

        if flavor_text.is_none() {
            warn!("no flavor texts provided in config :(");
        }

        flavor_text
    }
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct WatchersConfig {
    #[serde(default)]
    allow_by_default: bool,
    #[serde(default)]
    channels: Option<Vec<WatchersChannelConfig>>,
}

impl WatchersConfig {
    pub const fn allow_by_default(&self) -> bool {
        self.allow_by_default
    }

    pub const fn channels(&self) -> Option<&Vec<WatchersChannelConfig>> {
        self.channels.as_ref()
    }

    pub fn channel_allowed(&self, id: ChannelId) -> bool {
        self.channels().map_or_else(
            || self.allow_by_default(),
            |channels| {
                channels
                    .iter()
                    .find(|c| c.id == id)
                    .map_or_else(|| self.allow_by_default(), |channel| channel.allow)
            },
        )
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct WatchersChannelConfig {
    id: ChannelId,
    allow: bool,
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct BugReportsConfig {
    channel: Option<ChannelId>,
}

impl BugReportsConfig {
    fn channel(&self) -> Option<&ChannelId> {
        self.channel.as_ref()
    }
}

#[derive(Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct WordleConfig {
    //pub guesses_file: String,
    //pub answers_file: String,
    pub role_id: Option<RoleId>,
    pub channel_id: Option<ChannelId>,
}
