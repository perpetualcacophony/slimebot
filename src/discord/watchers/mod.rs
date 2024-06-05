use chrono::{DateTime, Utc};
use mongodb::{bson::doc, options::FindOneOptions, Database};
use poise::{
    serenity_prelude::{self, CacheHttp, Command, FullEvent, GuildId, Http, Message, UserId},
    FrameworkContext,
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use tokio::task::JoinSet;
use tracing::{info, instrument};

use crate::{
    data::PoiseData,
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

trait EventWatcher {
    type EventData;
    type FilterOutput;

    fn filter(event: &Self::EventData) -> Option<Self::FilterOutput>;
    async fn action(
        ctx: WatcherContext<'_>,
        filter: Self::FilterOutput,
        event: &Self::EventData,
    ) -> Result<(), CommandError>;

    async fn run(ctx: WatcherContext<'_>, event: &Self::EventData) -> Result<(), CommandError> {
        if let Some(filter) = Self::filter(event) {
            Self::action(ctx, filter, event).await
        } else {
            Ok(())
        }
    }
}

pub trait MessageWatcher {
    type FilterOutput = ();

    fn filter(msg: &Message) -> Option<Self::FilterOutput>;
    async fn action(
        ctx: WatcherContext<'_>,
        filter: Self::FilterOutput,
        msg: &Message,
    ) -> Result<(), CommandError>;
}

impl<T: MessageWatcher> EventWatcher for T {
    type EventData = Message;
    type FilterOutput = <Self as MessageWatcher>::FilterOutput;

    fn filter(event: &Self::EventData) -> Option<Self::FilterOutput> {
        <Self as MessageWatcher>::filter(event)
    }

    async fn action(
        ctx: WatcherContext<'_>,
        filter: Self::FilterOutput,
        event: &Self::EventData,
    ) -> Result<(), CommandError> {
        <Self as MessageWatcher>::action(ctx, filter, event).await
    }
}

pub struct VoreWatcher;
impl MessageWatcher for VoreWatcher {
    fn filter(msg: &Message) -> Option<()> {
        Regex::new(r"(?i)(?:[^a-z]|^)(voring|vores|vore)")
            .expect("hard-coded regex should be valid")
            .captures(&msg.content)
            .map(|_| ())
    }

    async fn action(ctx: WatcherContext<'_>, _: (), msg: &Message) -> Result<(), CommandError> {
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

        let vore_mentions = ctx.data().db.collection::<VoreMention>("vore_mentions");

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
                    ctx.cache_http(),
                    format!(
                        "~~{time}~~ 0 days without mentioning vore",
                        time = time.format_largest()
                    ),
                )
                .await?;
        }

        vore_mentions.insert_one(new_mention, None).await?;

        Ok(())
    }
}

pub struct BidenLWatcher;
impl MessageWatcher for BidenLWatcher {
    fn filter(msg: &Message) -> Option<()> {
        (&msg.content == "L").then_some(())
    }

    async fn action(ctx: WatcherContext<'_>, _: (), msg: &Message) -> Result<(), CommandError> {
        msg.channel_id
            .say_ext(ctx.cache_http(), include_str_static!("biden_L_url.txt"))
            .await?;

        Ok(())
    }
}

pub struct CLWatcher;
impl MessageWatcher for CLWatcher {
    fn filter(msg: &Message) -> Option<()> {
        msg.content
            .replace(['.', ',', ':', ';', '(', ')', '!', '?', '~', '#', '^'], " ")
            .split_ascii_whitespace()
            .any(|w| w == "CL")
            .then_some(())
    }

    async fn action(ctx: WatcherContext<'_>, _: (), msg: &Message) -> Result<(), CommandError> {
        let copypasta = include_str_static!("look_cl_copypasta.txt");

        if msg.content.starts_with("Look CL") || msg.content.starts_with("look CL") {
            msg.reply_ext(ctx.cache_http(), copypasta.trim_start_matches("Look CL, "))
                .await
                .map_err(SendMessageError::from)?;
        } else {
            msg.reply_ext(ctx.cache_http(), copypasta)
                .await
                .map_err(SendMessageError::from)?;
        }

        Ok(())
    }
}

pub struct HaikuWatcher;
impl MessageWatcher for HaikuWatcher {
    type FilterOutput = [String; 3];

    fn filter(msg: &Message) -> Option<Self::FilterOutput> {
        haiku::check_haiku(&msg.content)
    }

    async fn action(
        ctx: WatcherContext<'_>,
        haiku: Self::FilterOutput,
        msg: &Message,
    ) -> Result<(), CommandError> {
        let haiku = haiku
            .iter()
            .map(|line| format!("> *{line}*"))
            .collect::<Vec<_>>()
            .join("\n");

        let txt = format!("beep boop! i found a haiku:\n{haiku}\nsometimes i make mistakes");

        msg.reply_ext(ctx.cache_http(), txt).await?;

        Ok(())
    }
}

pub async fn run_all(ctx: WatcherContext<'static>, msg: &'static Message) {
    let mut join = JoinSet::new();

    join.spawn(VoreWatcher::run(ctx, msg));
}

#[derive(Copy, Clone)]
pub struct WatcherContext<'a> {
    serenity_ctx: &'a serenity_prelude::Context,
    framework_ctx: FrameworkContext<'a, PoiseData, CommandError>,
}

impl<'a> WatcherContext<'a> {
    fn data(self) -> &'a PoiseData {
        self.framework_ctx.user_data
    }

    pub fn new(
        serenity_ctx: &'a serenity_prelude::Context,
        framework_ctx: FrameworkContext<'a, PoiseData, CommandError>,
    ) -> Self {
        Self {
            serenity_ctx,
            framework_ctx,
        }
    }

    fn cache_http(self) -> &'a impl CacheHttp {
        self.serenity_ctx
    }
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
fn check_vore(content: &str) -> bool {
    Regex::new(r"(?i)(?:[^a-z]|^)(voring|vores|vore)")
        .expect("hard-coded regex should be valid")
        .captures(content)
        .is_some()
}

// watches all channels for a mention of vore and responds with time statistics
#[instrument(skip_all, level = "trace")]
pub async fn vore(ctx: WatcherContext<'_>, msg: &Message) -> Result<(), CommandError> {
    if check_vore(&msg.content) {
        log_watcher(ctx.cache_http(), msg).await;

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

        let vore_mentions = ctx.data().db.collection::<VoreMention>("vore_mentions");

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
                    ctx.cache_http(),
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
pub async fn l_biden(ctx: WatcherContext<'_>, msg: &Message) -> Result<(), CommandError> {
    if msg.content == "L" {
        info!(
            "@{} (#{}): {}",
            msg.author.name,
            msg.channel(ctx.cache_http())
                .await?
                .guild()
                .expect("channel should be inside a guild")
                .name(),
            msg.content
        );

        msg.channel_id
            .say_ext(ctx.cache_http(), include_str_static!("biden_L_url.txt"))
            .await?;
    }

    Ok(())
}

//#[cfg(feature = "look_cl")]
/// Watches all channels for "CL" and reponds with the Look CL copypasta.
#[instrument(skip_all, level = "trace")]
pub async fn look_cl(ctx: WatcherContext<'_>, msg: &Message) -> Result<(), CommandError> {
    if msg
        .content
        .replace(['.', ',', ':', ';', '(', ')', '!', '?', '~', '#', '^'], " ")
        .split_ascii_whitespace()
        .any(|w| w == "CL")
    {
        info!(
            "@{} (#{}): {}",
            msg.author.name,
            msg.channel(ctx.cache_http())
                .await
                .expect("message should be in a channel")
                .guild()
                .expect("channel should be in a guild")
                .name(),
            msg.content
        );

        let copypasta = include_str_static!("look_cl_copypasta.txt");

        if msg.content.starts_with("Look CL") || msg.content.starts_with("look CL") {
            msg.reply_ext(ctx.cache_http(), copypasta.trim_start_matches("Look CL, "))
                .await
                .map_err(SendMessageError::from)?;
        } else {
            msg.reply_ext(ctx.cache_http(), copypasta)
                .await
                .map_err(SendMessageError::from)?;
        }
    }

    Ok(())
}

#[instrument(skip_all)]
pub async fn watch_haiku(ctx: WatcherContext<'_>, msg: &Message) -> Result<(), CommandError> {
    if let Some(haiku) = haiku::check_haiku(&msg.content) {
        let haiku = haiku
            .iter()
            .map(|line| format!("> *{line}*"))
            .collect::<Vec<_>>()
            .join("\n");

        let txt = format!("beep boop! i found a haiku:\n{haiku}\nsometimes i make mistakes");

        msg.reply_ext(ctx.cache_http(), txt).await?;
    }

    Ok(())
}
