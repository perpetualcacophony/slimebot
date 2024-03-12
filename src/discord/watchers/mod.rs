use chrono::{DateTime, Utc};
use mongodb::{bson::doc, options::FindOneOptions, Database};
use poise::serenity_prelude::{CacheHttp, CreateMessage, Http, Message, UserId};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{debug, info, instrument};

use crate::FormatDuration;

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
pub async fn vore(http: &Http, db: &Database, new_message: &Message) {
    if check_vore(&new_message.content).await {
        let recent = Utc::now();

        log_watcher(http, new_message).await;

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
        vore_mentions
            .insert_one(new_mention, None)
            .await
            .expect("inserting to db should not fail");

        // fixing bug where error happens if collection has 1 object and returns none
        let last = if vore_mentions
            .count_documents(None, None)
            .await
            .expect("collection should have at least one item")
            == 1
        {
            vore_mentions
                .find_one(None, None)
                .await
                .expect("db request should not fail")
                .expect("collection should not be empty")
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
                .expect("db request should not fail")
                .expect("collection should not be empty")
                .timestamp
        };

        let time = recent - last;
        let time_text = time.format_largest();

        http.send_message(
            new_message.channel_id,
            Vec::new(),
            &json!({
                "content": format!("~~{time_text}~~ 0 days without mentioning vore")
            }),
        )
        .await
        .expect("sending message should not fail");
    }
}

// watches all channels for "L" and responds with the biden image
#[instrument(skip_all, level = "trace")]
pub async fn l_biden(http: &Http, new_message: &Message) {
    if new_message.content == "L" {
        info!(
            "@{} (#{}): {}",
            new_message.author.name,
            new_message
                .channel(http)
                .await
                .expect("message should have a channel")
                .guild()
                .expect("channel should be inside a guild")
                .name(),
            new_message.content
        );

        new_message
            .channel_id
            .send_message(
                http,
                CreateMessage::new().content("https://files.catbox.moe/v7itt0.webp"),
            )
            .await
            .expect("sending message should not fail");
    }
}

// watches all channels for "CL" and reponds with the Look CL copypasta
#[instrument(skip_all, level = "trace")]
pub async fn look_cl(http: &Http, new_message: &Message) {
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
                .channel(http)
                .await
                .expect("message should be in a channel")
                .guild()
                .expect("channel should be in a guild")
                .name(),
            new_message.content
        );

        if new_message.content.starts_with("Look CL") || new_message.content.starts_with("look CL")
        {
            new_message.channel_id.send_message(http, CreateMessage::new()
                .content("I wouldn't have wasted my time critiquing if I didn't think anafublic was a good writer. I would love to get feedback like this. Praise doesn't help you grow and I shared my honest impression as a reader with which you seem to mostly agree. As for my \"preaching post,\" I don't accept the premise that only ones bettors are qualified to share their opinion. Siskel and Ebert didn't know jack about making movies. As for me being \"lazy,\" that's the point. Reading shouldn't have to be work. If it is, you're doing something wrong. And I'm not being an asshole, I'm simply being direct.")
                .reference_message(new_message)
            )
            .await
            .expect("sending message should not fail");
        } else {
            new_message.channel_id.send_message(http, CreateMessage::new()
                .content("Look CL, I wouldn't have wasted my time critiquing if I didn't think anafublic was a good writer. I would love to get feedback like this. Praise doesn't help you grow and I shared my honest impression as a reader with which you seem to mostly agree. As for my \"preaching post,\" I don't accept the premise that only ones bettors are qualified to share their opinion. Siskel and Ebert didn't know jack about making movies. As for me being \"lazy,\" that's the point. Reading shouldn't have to be work. If it is, you're doing something wrong. And I'm not being an asshole, I'm simply being direct.")
                .reference_message(new_message)
            )
            .await
            .expect("sending message should not fail");
        }
    }
}

#[instrument(skip_all)]
pub async fn watch_haiku(http: &Http, msg: &Message) {
    if let Some(haiku) = haiku::check_haiku(&msg.content) {
        let haiku = haiku.iter().map(|line| format!("> *{line}*")).collect::<Vec<_>>().join("\n");

        let txt = format!("beep boop! i found a haiku:\n{haiku}\nsometimes i make mistakes");

        msg.reply(http, txt).await.unwrap();
    }
}