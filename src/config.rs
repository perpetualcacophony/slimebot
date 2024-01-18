use poise::serenity_prelude::{ChannelId, GuildId, UserId, Activity};
use serde::Deserialize;
use tracing::{info, warn, error, debug};
use tracing_unwrap::OptionExt;

use crate::DiscordToken;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub bot: BotConfig,
    pub logs: LogsConfig,
    pub db: DbConfig,
    pub watchers: WatchersConfig,
}

#[derive(Deserialize, Debug, Clone)]
pub struct BotConfig {
    token: Option<DiscordToken>,
    id: Option<UserId>,
    pub testing_server: Option<GuildId>,
    activity: Option<String>,
    pub status: Option<String>,
    prefix: String
}

impl BotConfig {
    pub fn token(&self) -> &str {
        self.token
            .as_ref()
            .expect_or_log("no token in config or environment!")
    }

    pub fn id(&self) -> UserId {
        self.id
            .expect_or_log("no user id in config or environment!")
    }

    pub fn activity(&self) -> Option<Activity> {
        if let Some(activity) = &self.activity {
            if activity.is_empty() {
                warn!("bot.activity provided in config as empty string, defaulting to none");
                return None;
            }

            let parsed_activity = if activity.starts_with("playing ") {
                Activity::playing(activity.strip_prefix("playing ").unwrap())
            } else if activity.starts_with("listening to ") {
                Activity::playing(activity.strip_prefix("listening to ").unwrap())
            } else if activity.starts_with("watching ") {
                Activity::playing(activity.strip_prefix("watching ").unwrap())
            } else if activity.starts_with("competing in ") {
                Activity::playing(activity.strip_prefix("competing in ").unwrap())
            } else {
                error!("bot.activity in config could not be parsed - must start with `playing`, `listening to`, `watching` or `competing in`");
                warn!("disabling bot activity");
                return None;
            };

            debug!("bot.activity parsed as {:?}: {}", parsed_activity.kind, parsed_activity.name);
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

    pub fn prefix(&self) -> &str {
        &self.prefix
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct LogsConfig {
    pub discord: DiscordConfig,
}

#[derive(Deserialize, Debug, Clone)]
pub struct DiscordConfig {
    enabled: bool,
    channel: Option<ChannelId>,
}

impl DiscordConfig {
    pub fn enabled(&self) -> bool {
        if self.enabled {
            info!("discord logger enabled");
            true
        } else {
            false
        }
    }

    pub fn channel(&self) -> Option<ChannelId> {
        if self.enabled {
            match self.channel {
                Some(_) => Some(self.channel.unwrap()),
                None => {
                    warn!("no channel configured for discord logger");
                    None
                }
            }
        } else {
            None
        }
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
    pub fn allow_by_default(&self) -> bool {
        self.allow_by_default
    }

    pub fn channels(&self) -> Option<&Vec<WatchersChannelConfig>> {
        self.channels.as_ref()
    }

    pub fn channel_allowed(&self, id: ChannelId) -> bool {
        if let Some(channels) = self.channels() {
            if let Some(channel) = channels.iter().find(|c| c.id == id) {
                channel.allow
            } else {
                self.allow_by_default()
            }
        } else {
            self.allow_by_default()
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct WatchersChannelConfig {
    id: ChannelId,
    allow: bool,
}
