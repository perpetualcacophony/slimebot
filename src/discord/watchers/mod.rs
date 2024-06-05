use chrono::{DateTime, Utc};
use mongodb::{bson::doc, options::FindOneOptions, Database};
use poise::serenity_prelude::{CacheHttp, GuildId, Http, Message, UserId};
use regex::Regex;
use serde::{Deserialize, Serialize};
use tracing::{info, instrument};

use crate::{
    errors::CommandError,
    utils::{
        format_duration::FormatDuration,
        serenity::channel::{ChannelIdExt, MessageExt},
    },
};

macro_rules! include_str_static {
    ($path:literal) => {
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/static/", $path))
    };
}

use super::commands::SendMessageError;

mod haiku;

async fn log_watcher(http: impl CacheHttp, new_message: &Message) {
    info!(
        "@{} (#{}): {}",
        new_message.author.name,
        new_message
            .channel(http)
            .await
            .expect("message should have channel")
            .guild()
            .expect("channel should be in a guild")
            .name(),
        new_message.content
    );
}

#[instrument(skip_all, level = "trace")]
async fn check_vore(content: &str) -> bool {
    Regex::new(r"(?i)(?:[^a-z]|^)(voring|vores|vore)")
        .expect("hard-coded regex should be valid")
        .captures(content)
        .is_some()
}

// watches all channels for a mention of vore and responds with time statistics
#[instrument(skip_all, level = "trace")]
pub async fn vore(http: &Http, db: &Database, msg: &Message) -> Result<(), CommandError> {
    if check_vore(&msg.content).await {
        log_watcher(http, msg).await;

        #[derive(Debug, Deserialize, Serialize)]
        struct VoreMention {
            timestamp: DateTime<Utc>,
            author: UserId,
            guild: GuildId,
        }

        let new_mention = VoreMention {
            timestamp: Utc::now(),
            author: msg.author.id,
            guild: msg.guild_id.expect("message should be in guild"),
        };

        let vore_mentions = db.collection::<VoreMention>("vore_mentions");

        let guild_bson =
            mongodb::bson::ser::to_bson(&new_mention.guild).expect("GuildId can be serialized");

        if let Some(last) = vore_mentions
            .find_one(
                doc! { "guild": guild_bson },
                FindOneOptions::builder()
                    .sort(doc! { "timestamp": -1 })
                    .build(),
            )
            .await?
        {
            let time = new_mention.timestamp - last.timestamp;

            msg.channel_id
                .say_ext(
                    http,
                    format!(
                        "~~{time}~~ 0 days without mentioning vore",
                        time = time.format_largest()
                    ),
                )
                .await?;
        }

        vore_mentions.insert_one(new_mention, None).await?;
    }

    Ok(())
}

// watches all channels for "L" and responds with the biden image
#[instrument(skip_all, level = "trace")]
pub async fn l_biden(http: &Http, msg: &Message) -> Result<(), CommandError> {
    if msg.content == "L" {
        info!(
            "@{} (#{}): {}",
            msg.author.name,
            msg.channel(http)
                .await?
                .guild()
                .expect("channel should be inside a guild")
                .name(),
            msg.content
        );

        msg.channel_id
            .say_ext(http, include_str_static!("biden_L_url.txt"))
            .await?;
    }

    Ok(())
}

#[cfg(feature = "look_cl")]
/// Watches all channels for "CL" and reponds with the Look CL copypasta.
#[instrument(skip_all, level = "trace")]
pub async fn look_cl(http: &Http, msg: &Message) -> Result<(), CommandError> {
    if msg
        .content
        .replace(['.', ',', ':', ';', '(', ')', '!', '?', '~', '#', '^'], " ")
        .split_ascii_whitespace()
        .any(|w| w == "CL")
    {
        info!(
            "@{} (#{}): {}",
            msg.author.name,
            msg.channel(http)
                .await
                .expect("message should be in a channel")
                .guild()
                .expect("channel should be in a guild")
                .name(),
            msg.content
        );

        let copypasta = include_str_static!("look_cl_copypasta.txt");

        if msg.content.starts_with("Look CL") || msg.content.starts_with("look CL") {
            msg.reply_ext(http, copypasta.trim_start_matches("Look CL, "))
                .await
                .map_err(SendMessageError::from)?;
        } else {
            msg.reply_ext(http, copypasta)
                .await
                .map_err(SendMessageError::from)?;
        }
    }

    Ok(())
}

#[instrument(skip_all)]
pub async fn watch_haiku(http: &Http, msg: &Message) -> Result<(), CommandError> {
    if let Some(haiku) = haiku::check_haiku(&msg.content) {
        let haiku = haiku
            .iter()
            .map(|line| format!("> *{line}*"))
            .collect::<Vec<_>>()
            .join("\n");

        let txt = format!("beep boop! i found a haiku:\n{haiku}\nsometimes i make mistakes");

        msg.reply_ext(http, txt).await?;
    }

    Ok(())
}
