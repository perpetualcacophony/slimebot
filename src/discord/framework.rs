use chrono::Utc;
use poise::serenity_prelude as serenity;
use serde_json::json;

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

        if new_message.author.id != framework_data.bot_id {
            if new_message.content == "L" {
                ctx.http.send_message(
                    new_message.channel_id.into(),
                    &json!({
                        "content": "https://cdn.discordapp.com/attachments/1126687533900771429/1149042466327109814/IMG_3244.webp?ex=65b15130&is=659edc30&hm=189463085657b1bf66f7ea9daf5b341dc16a53c8485e6b1aa55705a2a22522c6&"
                    })
                ).await.unwrap();
            } else if new_message
                .content
                .to_lowercase()
                // this code sucks less ass.
                .replace([',', ':', ';', '(', ')', '!', '?', '~'], " ")
                .split_ascii_whitespace()
                .any(|w| w == "vore")
            {
                let more_data: MoreData = serde_json::from_str(&tokio::fs::read_to_string("slimebot_data.json").await.unwrap()).unwrap();

                let recent = Utc::now();
                let last = more_data.last_vore_mention;
                let time = recent - last;

                let mut time_text = String::new();
                let _ = time_text.is_ascii();
                if time.num_days() >= 1 {
                    time_text = format!("{} day", time.num_days());
                    if time.num_days() > 1 {
                        time_text.push('s')
                    }
                } else if time.num_hours() >= 1 {
                    time_text = format!("{} hour", time.num_hours());
                    if time.num_hours() > 1 {
                        time_text.push('s')
                    }
                } else if time.num_minutes() >= 1 {
                    time_text = format!("{} minute", time.num_minutes());
                    if time.num_minutes() > 1 {
                        time_text.push('s')
                    }
                } else {
                    time_text = format!("{} second", time.num_seconds());
                    if time.num_seconds() > 1 {
                        time_text.push('s')
                    }
                }

                ctx.http
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
                tokio::fs::write("slimebot_data.json", &serde_json::to_string(&more_data).unwrap().as_bytes()).await.unwrap();
            }
        }

        poise::dispatch_event(framework_data, &ctx, &poise::Event::Message { new_message }).await;
    }

    /*async fn interaction_create(
        &self,
        ctx: serenity::Context,
        interaction: Interaction
    ) {
        let shard_manager = (*self.shard_manager.lock().unwrap()).clone().unwrap();
        let framework = poise::FrameworkContext {
            bot_id: serenity::UserId(846453852164587620),
            options: &self.options,
            user_data: &(),
            shard_manager: &shard_manager
        };

        poise::dispatch_interaction(framework, &ctx, interaction.as_application_command().unwrap(), &AtomicBool::new(true), invocation_data, parent_commands).await;
    }*/
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
