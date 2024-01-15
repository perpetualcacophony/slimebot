use std::sync::atomic::AtomicBool;

use chrono::Utc;
use poise::serenity_prelude::{self as serenity, CacheHttp, Interaction};
use serde_json::json;
use tokio::sync::Mutex;
use tracing::info;

use crate::{Data, Error, MoreData};

pub struct Handler {
    pub data: Data,
    pub options: poise::FrameworkOptions<Data, Error>,
    pub shard_manager:
        std::sync::Mutex<Option<std::sync::Arc<tokio::sync::Mutex<serenity::ShardManager>>>>,
}

#[serenity::async_trait]
impl serenity::EventHandler for Handler {
    async fn message(&self, ctx: serenity::Context, new_message: serenity::Message) {
        let shard_manager = (*self.shard_manager.lock().unwrap()).clone().unwrap();
        let framework_data = poise::FrameworkContext {
            bot_id: self.data.config.bot.id(),
            options: &self.options,
            user_data: &self.data,
            shard_manager: &shard_manager,
        };

        if new_message.author.id != framework_data.bot_id && !new_message.is_private() {
            if new_message.content == "L" {
                ctx.http().send_message(
                    new_message.channel_id.into(),
                    &json!({
                        "content": "https://cdn.discordapp.com/attachments/1126687533900771429/1149042466327109814/IMG_3244.webp?ex=65b15130&is=659edc30&hm=189463085657b1bf66f7ea9daf5b341dc16a53c8485e6b1aa55705a2a22522c6&"
                    })
                ).await.unwrap();
            } else if new_message
                .content
                .to_lowercase()
                // this code sucks less ass.
                .replace(['.', ',', ':', ';', '(', ')', '!', '?', '~'], " ")
                .split_ascii_whitespace()
                .any(|w| w == "vore" || w == "voring")
            {
                let more_data: MoreData = serde_json::from_str(
                    &tokio::fs::read_to_string("slimebot_data.json")
                        .await
                        .unwrap(),
                )
                .unwrap();

                let recent = Utc::now();
                let last = more_data.last_vore_mention;
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

                let mut more_data = self.data.more_data.clone();
                more_data.last_vore_mention = recent;
                tokio::fs::write(
                    "slimebot_data.json",
                    &serde_json::to_string(&more_data).unwrap().as_bytes(),
                )
                .await
                .unwrap();
            } else if new_message
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

                if new_message.content.starts_with("Look CL")
                    || new_message.content.starts_with("look CL")
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

        poise::dispatch_event(framework_data, &ctx, &poise::Event::Message { new_message }).await;
    }

    async fn interaction_create(&self, ctx: serenity::Context, interaction: Interaction) {
        let shard_manager = (*self.shard_manager.lock().unwrap()).clone().unwrap();
        let framework = poise::FrameworkContext {
            bot_id: serenity::UserId(846453852164587620),
            options: &self.options,
            user_data: &self.data,
            shard_manager: &shard_manager,
        };

        poise::dispatch_interaction(
            framework,
            &ctx,
            interaction.as_application_command().unwrap(),
            &AtomicBool::new(false),
            &Mutex::new(Box::new(())), // literally no idea what `invocation_data` is supposed to be LOL
            &mut self.options.commands.iter().collect(),
        )
        .await
        .ok();
    }
}

/*pub async fn manual_dispatch(token: String) {
    let intents = GatewayIntents::all();
    let mut handler = Handler {
        options: FrameworkOptions {
            commands: vec![ping()],
            ..Default::default()
        },
        shard_manager: Mutex::new(None)
    };
    poise::set_qualified_names(&mut handler.options.commands);

    let handler = Arc::new(handler);
    let mut client = Client::builder(token, intents)
        .event_handler_arc(handler.clone())
        .register_songbird()
        .await
        .unwrap();
}*/
