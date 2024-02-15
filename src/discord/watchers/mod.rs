use std::{future::Future, pin::Pin, sync::Arc};

use chrono::{DateTime, Utc};
use mongodb::{bson::doc, options::FindOneOptions, Database};
use poise::serenity_prelude::{collect, futures::{future, Stream, StreamExt}, CacheHttp, Context, CreateMessage, Event, Message, MessageCollector, UserId};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{info, instrument};

use crate::{Data, FormatDuration};

async fn log_watcher(ctx: &Context, new_message: &Message) {
    info!(
        "@{} (#{}): {}",
        new_message.author.name,
        new_message
            .channel(ctx.http())
            .await
            .unwrap() // todo: handle the http request failing
            .guild()
            .unwrap() // this is ok - the message will not be outside a guild
            .name(),
        new_message.content
    );
}

trait Callback: Fn(&Context, &Data, Message) -> Self::Future {
    type Future: Future<Output = ()>;
}
impl<F, Fut> Callback for F
where 
    F: Fn(&Context, &Data, Message) -> Self::Future,
    Fut: Future<Output = ()>,
{
    type Future = Fut;
}

#[derive(Clone)]
struct MessageWatcher {
    regex: Regex,
    callback: Box<dyn Callback,
}

impl MessageWatcher {
    fn new(regex: &str, callback: Callback) -> Self {
        let regex = Regex::new(regex).unwrap();

        Self { regex, callback }
    }
}

pub struct Registry {
    ctx: Context,
    data: Data,
    collector: MessageCollector,
    watchers: Vec<MessageWatcher>,
}

impl Registry {
    pub fn new(ctx: Context, data: Data) -> Self {
        let collector = MessageCollector::new(ctx.shard.clone());

        Self { ctx, data, collector, watchers: Vec::new() }
    }

    pub fn add_watcher(mut self, regex: &str, callback: Callback) -> Self {
        self.watchers.push(MessageWatcher::new(regex, callback));

        self
    }

    pub fn run(self) {
        let messages = self.collector.stream(); 

        let watchers = self.watchers;
        let arc = Arc::new(watchers);

        let data_arc = Arc::new((self.ctx, self.data));

        tokio::spawn({
            messages.for_each(move |msg| {
                let data = data_arc.clone();
                let watchers = arc.clone();

                async move {
                    for watcher in watchers.iter() {
                        let callback = watcher.callback;

                        callback(&data.0, &data.1, msg.clone()).await;
                    }
                }
            })
        });
    }


    async fn call_watchers(self: Arc<Self>, msg: Message) {
        for watcher in &self.watchers {
            let callback = watcher.callback;
            callback(&self.ctx, &self.data, msg.clone()).await;
        }
    }
}

#[instrument(skip_all, level = "trace")]
async fn check_vore(content: &str) -> bool {
    Regex::new(r"(?i)(?:[^a-z]|^)(voring|vores|vore)")
        .unwrap()
        .captures(content)
        .is_some()
}

// watches all channels for a mention of vore and responds with time statistics
#[instrument(skip_all, level = "trace")]
pub async fn vore(ctx: &Context, db: &Database, new_message: &Message) {
    if check_vore(&new_message.content).await {
        let recent = Utc::now();

        log_watcher(ctx, new_message).await;

        #[derive(Debug, Deserialize, Serialize)]
        struct VoreMention {
            timestamp: DateTime<Utc>,
            author: UserId,
        }

        let vore_mentions = db.collection::<VoreMention>("vore_mentions");

        let new_mention = VoreMention {
            timestamp: recent,
            author: new_message.author.id,
        };
        vore_mentions.insert_one(new_mention, None).await.unwrap();

        // fixing bug where error happens if collection has 1 object and returns none
        let last = if vore_mentions.count_documents(None, None).await.unwrap() == 1 {
            vore_mentions
                .find_one(None, None)
                .await
                .unwrap() // will fail if db connection fails
                .unwrap() // will fail if collection is empty
                .timestamp
        } else {
            vore_mentions
                .find_one(
                    doc! { "timestamp": { "$ne": format!("{recent:?}") } },
                    FindOneOptions::builder()
                        .sort(doc! { "timestamp": -1 })
                        .build(),
                )
                .await
                .unwrap() // will fail if db connection fails
                .unwrap() // will fail if collection is empty
                .timestamp
        };

        let time = recent - last;
        let time_text = time.format_largest();

        ctx.http()
            .send_message(
                new_message.channel_id.into(),
                Vec::new(),
                &json!({
                    "content": format!("~~{time_text}~~ 0 days without mentioning vore")
                }),
            )
            .await
            .unwrap();
    }
}

// watches all channels for "L" and responds with the biden image
#[instrument(skip_all, level = "trace")]
pub async fn l_biden(ctx: &Context, new_message: &Message) {
    if new_message.content == "L" {
        info!(
            "@{} (#{}): {}",
            new_message.author.name,
            new_message
                .channel(ctx.http())
                .await
                .unwrap() // todo: handle the http request failing
                .guild()
                .unwrap() // this is ok - the message will not be outside a guild
                .name(),
            new_message.content
        );

        new_message
            .channel_id
            .send_message(
                ctx.http(),
                CreateMessage::new().content("https://files.catbox.moe/v7itt0.webp"),
            )
            .await
            .unwrap();
    }
}

// watches all channels for "CL" and reponds with the Look CL copypasta
#[instrument(skip_all, level = "trace")]
pub async fn look_cl(ctx: &Context, new_message: &Message) {
    if new_message
        .content
        .replace(['.', ',', ':', ';', '(', ')', '!', '?', '~', '#', '^'], " ")
        .split_ascii_whitespace()
        .any(|w| w == "CL")
    {
        info!(
            "@{} (#{}): {}",
            new_message.author.name,
            new_message
                .channel(ctx.http())
                .await
                .unwrap() // todo: handle the http request failing
                .guild()
                .unwrap() // this is ok - the message will not be outside a guild
                .name(),
            new_message.content
        );

        if new_message.content.starts_with("Look CL") || new_message.content.starts_with("look CL")
        {
            new_message.channel_id.send_message(ctx.http(), CreateMessage::new()
                .content("I wouldn't have wasted my time critiquing if I didn't think anafublic was a good writer. I would love to get feedback like this. Praise doesn't help you grow and I shared my honest impression as a reader with which you seem to mostly agree. As for my \"preaching post,\" I don't accept the premise that only ones bettors are qualified to share their opinion. Siskel and Ebert didn't know jack about making movies. As for me being \"lazy,\" that's the point. Reading shouldn't have to be work. If it is, you're doing something wrong. And I'm not being an asshole, I'm simply being direct.")
                .reference_message(new_message)
            )
            .await
            .unwrap();
        } else {
            new_message.channel_id.send_message(ctx.http(), CreateMessage::new()
                .content("Look CL, I wouldn't have wasted my time critiquing if I didn't think anafublic was a good writer. I would love to get feedback like this. Praise doesn't help you grow and I shared my honest impression as a reader with which you seem to mostly agree. As for my \"preaching post,\" I don't accept the premise that only ones bettors are qualified to share their opinion. Siskel and Ebert didn't know jack about making movies. As for me being \"lazy,\" that's the point. Reading shouldn't have to be work. If it is, you're doing something wrong. And I'm not being an asshole, I'm simply being direct.")
                .reference_message(new_message)
            )
            .await
            .unwrap();
        }
    }
}
