use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};

use poise::serenity_prelude::{ActivityData, ChannelId, GuildId, RoleId};
use rand::seq::IteratorRandom;
use serde::Deserialize;
use tracing::{debug, error, info, warn};

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub secrets_dir: Option<PathBuf>,

    pub bot: BotConfig,
    pub logs: LogsConfig,
    pub db: DbConfig,
    #[serde(default)]
    pub watchers: WatchersConfig,
    #[serde(default)]
    pub bug_reports: BugReportsConfig,

    #[cfg(feature = "wordle")]
    #[serde(default)]
    pub wordle: WordleConfig,
}

impl Config {
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

#[derive(Deserialize, Debug, Clone)]
pub struct BotConfig {
    testing_server: Option<GuildId>,
    activity: Option<String>,
    prefix: String,
    status_channel: Option<ChannelId>,
    github_repo: Option<RepoName>,
}

impl BotConfig {
    pub fn testing_server(&self) -> Option<&GuildId> {
        if self.testing_server.is_none() {
            warn!("no testing server set in config, slash commands will not be registered");
        }

        self.testing_server.as_ref()
    }

    pub fn activity(&self) -> Option<ActivityData> {
        if let Some(activity) = &self.activity {
            if activity.is_empty() {
                warn!("bot.activity provided in config as empty string, defaulting to none");
                return None;
            }

            let parsed_activity = if activity.starts_with("playing ") {
                ActivityData::playing(
                    activity
                        .strip_prefix("playing ")
                        .expect("activity should have prefix"),
                )
            } else if activity.starts_with("listening to ") {
                ActivityData::playing(
                    activity
                        .strip_prefix("listening to ")
                        .expect("activity should have prefix"),
                )
            } else if activity.starts_with("watching ") {
                ActivityData::playing(
                    activity
                        .strip_prefix("watching ")
                        .expect("activity should have prefix"),
                )
            } else if activity.starts_with("competing in ") {
                ActivityData::playing(
                    activity
                        .strip_prefix("competing in ")
                        .expect("activity should have prefix"),
                )
            } else {
                error!("bot.activity in config could not be parsed - must start with `playing`, `listening to`, `watching` or `competing in`");
                warn!("disabling bot activity");
                return None;
            };

            debug!(
                "bot.activity parsed as {:?}: {}",
                parsed_activity.kind, parsed_activity.name
            );
            info!("successfully parsed bot activity from config");

            Some(parsed_activity)
        } else {
            warn!("no bot.activity provided in config, defaulting to none");
            None
        }
    }

    pub fn prefix(&self) -> &str {
        &self.prefix
    }

    pub fn status_channel(&self) -> Option<ChannelId> {
        self.status_channel
    }

    pub fn github_repo(&self) -> Option<&RepoName> {
        if self.github_repo.is_none() {
            tracing::warn!("no github repository in config");
        }

        self.github_repo.as_ref()
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(try_from = "String")]
pub struct RepoName {
    user: String,
    repo: String,
}

impl std::fmt::Display for RepoName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.user, self.repo)
    }
}

impl RepoName {
    fn new(user: String, repo: String) -> Self {
        Self { user, repo }
    }

    pub fn try_from_str(s: &str) -> Option<Self> {
        let (user, repo) = s.split_once('/')?;
        Some(Self::new(user.to_owned(), repo.to_owned()))
    }

    pub fn to_github_url(&self) -> reqwest::Url {
        let github = reqwest::Url::parse("https://github.com").expect("github is a valid url");
        let mut url = github;
        url.set_path(&self.to_string());
        url
    }
}

impl TryFrom<String> for RepoName {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from_str(&value).ok_or(value)
    }
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

#[derive(Deserialize, Debug, Clone)]
pub struct DbConfig {
    url: String,
}

impl DbConfig {
    pub fn url(&self) -> Cow<str> {
        #[cfg(feature = "docker")]
        if let Ok(db_url) = std::env::var("SLIMEBOT_DB_URL") {
            tracing::trace!(db_url, "using db url override from environment");
            return db_url.into();
        }

        (&self.url).into()
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
