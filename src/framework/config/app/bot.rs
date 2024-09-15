use poise::serenity_prelude::{ActivityData, ChannelId, GuildId};
use serde::Deserialize;
use tracing::{debug, error, info, warn};

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
