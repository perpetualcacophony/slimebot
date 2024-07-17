use poise::serenity_prelude::{ActivityData, ChannelId, GuildId, RoleId};
use rand::seq::IteratorRandom;
use serde::Deserialize;
use tracing::{debug, error, info, warn};
use tracing_unwrap::OptionExt;

use crate::DiscordToken;

#[derive(Debug, Clone, thiserror::Error)]
pub enum ConfigError {
    #[error(transparent)]
    Bot(#[from] BotConfigError),
}

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub bot: BotConfig,
    pub logs: LogsConfig,
    pub db: DbConfig,
    #[serde(default)]
    pub watchers: WatchersConfig,
    #[serde(default)]
    pub bug_reports: BugReportsConfig,
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
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum BotConfigError {
    #[error("no github repository in config")]
    GithubRepo,
}

#[derive(Deserialize, Debug, Clone)]
pub struct BotConfig {
    token: Option<DiscordToken>,
    testing_server: Option<GuildId>,
    activity: Option<String>,
    prefix: String,
    status_channel: Option<ChannelId>,
    github_repo: Option<RepoName>,
}

impl BotConfig {
    pub fn token(&self) -> &str {
        self.token
            .as_ref()
            .expect_or_log("no token in config or environment!")
    }

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

    pub fn github_repo(&self) -> Result<&RepoName, BotConfigError> {
        self.github_repo.as_ref().ok_or(BotConfigError::GithubRepo)
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
        let github = reqwest::Url::parse("https://github.com").unwrap();
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
    username: String,
    password: String,
}

impl DbConfig {
    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn password(&self) -> &str {
        &self.password
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
