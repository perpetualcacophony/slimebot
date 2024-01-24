use poise::{
    samples::register_in_guild,
    serenity_prelude::{
        self as serenity, CacheHttp, Context, Interaction, Reaction, Ready, UserId,
    },
};
use std::sync::atomic::AtomicBool;
use tokio::sync::Mutex;
use tracing::trace;

use crate::{BotError, Data};

use super::bug_reports::bug_reports;

pub struct Handler {
    pub data: Data,
    pub options: poise::FrameworkOptions<Data, BotError>,
    pub shard_manager:
        std::sync::Mutex<Option<std::sync::Arc<tokio::sync::Mutex<serenity::ShardManager>>>>,
}

#[serenity::async_trait]
impl serenity::EventHandler for Handler {
    async fn message(&self, ctx: serenity::Context, new_message: serenity::Message) {
        let shard_manager = (*self.shard_manager.lock().unwrap()).clone().unwrap();
        let framework_data = poise::FrameworkContext {
            bot_id: UserId(ctx.http().http().application_id().unwrap()),
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
            bot_id: UserId(ctx.http().http().application_id().unwrap()),
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

        if let Some(guild) = self.data.config.bot.testing_server() {
            register_in_guild(ctx.http(), self.options.commands.as_ref(), *guild)
                .await
                .unwrap();
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

    async fn reaction_add(&self, ctx: Context, add_reaction: Reaction) {
        if let Some(channel_id) = self.data.config().bug_reports_channel() {
            bug_reports(&ctx, add_reaction, channel_id).await;
        }
    }
}
