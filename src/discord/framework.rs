use poise::serenity_prelude::{self as serenity, CacheHttp, Interaction, Ready};
use std::sync::atomic::AtomicBool;
use tokio::sync::Mutex;
use tracing::trace;

use crate::{Data, Error};

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

        if new_message.author.id != framework_data.bot_id
            && !new_message.is_private()
            && self
                .data
                .config()
                .watchers
                .channel_allowed(new_message.channel_id)
        {
            #[allow(clippy::wildcard_imports)]
            use super::watchers::*;
            tokio::join!(
                vore(&ctx, self.data.db(), &new_message),
                look_cl(&ctx, &new_message),
                l_biden(&ctx, &new_message),
            );
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

    async fn ready(&self, ctx: serenity::Context, _: Ready) {
        trace!("Ready event received from discord!");

        if let Some(activity) = self.data.config().bot.activity() {
            ctx.set_activity(activity).await;
        }

        let mut keep_alive = tokio::time::interval(std::time::Duration::from_secs(600));
        keep_alive.tick().await;
        loop {
            let before = std::time::SystemTime::now();
            if ctx.http().get_bot_gateway().await.is_err() {
                tracing::error!("failed to connect to discord!");
            }
            let ping = std::time::SystemTime::now()
                .duration_since(before)
                .unwrap()
                .as_millis();
            tracing::info!("discord connection active! ({ping}ms)");

            keep_alive.tick().await;
        }
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
