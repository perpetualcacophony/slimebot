use poise::serenity_prelude::{ChannelId, GuildId, UserId};
use serde::Deserialize;
use tracing::{info, warn};
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
