use chrono::{DateTime, Utc};
use mongodb::{bson::doc, options::FindOneOptions};
use poise::serenity_prelude::{CacheHttp, Context, Message, UserId};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::info;

use super::framework::Handler;

// watches all channels for a mention of vore and responds with time statistics
pub async fn vore(ctx: &Context, handler: &Handler, new_message: &Message) {
    if new_message
        .content
        .to_lowercase()
        // this code sucks less ass.
        .replace(['.', ',', ':', ';', '(', ')', '!', '?', '~'], " ")
        .split_ascii_whitespace()
        .any(|w| w == "vore" || w == "voring" || w == "vores")
    {
        let recent = Utc::now();

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

        #[derive(Debug, Deserialize, Serialize)]
        struct VoreMention {
            timestamp: DateTime<Utc>,
            author: UserId,
        }

        let db = handler.data.db();
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

        let (d, h, m, s) = (
            time.num_days(),
            time.num_hours(),
            time.num_minutes(),
            time.num_seconds(),
        );

        let time_text = match (d, h, m, s) {
            (1, _, _, _) => ("1 day").to_string(),
            (2.., _, _, _) => format!("{d} days"),
            (_, 1, _, _) => ("1 hour").to_string(),
            (_, 2.., _, _) => format!("{h} hours"),
            (_, _, 1, _) => ("1 minute").to_string(),
            (_, _, 2.., _) => format!("{m} minutes"),
            (_, _, _, 1) => ("1 second").to_string(),
            (_, _, _, 2..) => format!("{s} seconds"),
            (_, _, _, _) => "less than a second".to_string(),
        };

        ctx.http()
            .send_message(
                new_message.channel_id.into(),
                &json!({
                    "content": format!("~~{time_text}~~ 0 days without mentioning vore")
                }),
            )
            .await
            .unwrap();
    }
}

// watches all channels for "L" and responds with the biden image
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

        ctx.http().send_message(
            new_message.channel_id.into(),
            &json!({
                "content": "https://files.catbox.moe/v7itt0.webp"
            })
        ).await.unwrap();
    }
}

// watches all channels for "CL" and reponds with the Look CL copypasta
pub async fn look_cl(ctx: &Context, new_message: &Message) {
    if new_message
        .content
        .replace(['.', ',', ':', ';', '(', ')', '!', '?', '~'], " ")
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
            ctx.http()
            .send_message(
                new_message.channel_id.into(),
                &json!({
                    "content": "I wouldn't have wasted my time critiquing if I didn't think anafublic was a good writer. I would love to get feedback like this. Praise doesn't help you grow and I shared my honest impression as a reader with which you seem to mostly agree. As for my \"preaching post,\" I don't accept the premise that only ones bettors are qualified to share their opinion. Siskel and Ebert didn't know jack about making movies. As for me being \"lazy,\" that's the point. Reading shouldn't have to be work. If it is, you're doing something wrong. And I'm not being an asshole, I'm simply being direct."
                }),
            )
            .await
            .unwrap();
        } else {
            ctx.http()
            .send_message(
                new_message.channel_id.into(),
                &json!({
                    "content": "Look CL, I wouldn't have wasted my time critiquing if I didn't think anafublic was a good writer. I would love to get feedback like this. Praise doesn't help you grow and I shared my honest impression as a reader with which you seem to mostly agree. As for my \"preaching post,\" I don't accept the premise that only ones bettors are qualified to share their opinion. Siskel and Ebert didn't know jack about making movies. As for me being \"lazy,\" that's the point. Reading shouldn't have to be work. If it is, you're doing something wrong. And I'm not being an asshole, I'm simply being direct."
                }),
            )
            .await
            .unwrap();
        }
    }
}
