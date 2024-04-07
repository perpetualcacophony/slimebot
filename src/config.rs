use poise::serenity_prelude::{ActivityData, ChannelId, GuildId, RoleId};
use rand::seq::IteratorRandom;
use serde::Deserialize;
use tracing::{debug, error, info, warn};
use tracing_unwrap::OptionExt;

use crate::DiscordToken;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub bot: BotConfig,
    pub logs: LogsConfig,
    pub db: DbConfig,
    pub watchers: WatchersConfig,
    pub bug_reports: Option<BugReportsConfig>,
    pub wordle: Option<WordleConfig>,
}

impl Config {
    pub fn bug_reports_channel(&self) -> Option<&ChannelId> {
        if let Some(bug_reports_config) = &self.bug_reports {
            Some(bug_reports_config.channel())
        } else {
            warn!("bug reports not configured");
            None
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct BotConfig {
    token: Option<DiscordToken>,
    testing_server: Option<GuildId>,
    activity: Option<String>,
    prefix: String,
    status_channel: Option<ChannelId>,
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

#[derive(Deserialize, Debug, Clone)]
pub struct WatchersConfig {
    allow_by_default: bool,
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

#[derive(Deserialize, Debug, Clone)]
pub struct BugReportsConfig {
    channel: ChannelId,
}

impl BugReportsConfig {
    fn channel(&self) -> &ChannelId {
        &self.channel
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct WordleConfig {
    //pub guesses_file: String,
    //pub answers_file: String,
    pub role_id: RoleId,
    pub channel_id: ChannelId,
}
