use poise::{
    samples::register_in_guild,
    serenity_prelude::{
        self as serenity, CacheHttp, Context, Interaction, Reaction, ReactionType,
        Ready, UserId, CreateEmbed, Color, futures::future::join_all,
    },
};
use std::sync::atomic::AtomicBool;
use tokio::sync::Mutex;
use tracing::{trace, debug};

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
        if add_reaction.emoji == ReactionType::Unicode("🐞".to_string()) {
            let messages = add_reaction
                .channel_id
                .messages(ctx.http(), |get| {
                    get.around(add_reaction.message_id).limit(5)
                })
                .await
                .unwrap();

            let http = ctx.http();

            let messages = messages.into_iter()
                .rev()
                .enumerate()
                .map(|(n, m)| async move {
                    let content = if m.content.is_empty() {
                        "*empty message*"
                    } else {
                        &m.content
                    };

                    let name = m.author_nick(http).await.unwrap_or(m.author.name);

                    if n == 2 {
                        (
                            format!(
                                "{name} << bug occurred here {}",
                                add_reaction.message_id.link(add_reaction.channel_id, add_reaction.guild_id)),
                            format!("**{}**", content),
                            false
                        )
                    } else {
                        (name, content.to_string(), false)
                    }
                });

            let messages = join_all(messages).await;

            debug!("{:#?}", messages);

            let mut embed = CreateEmbed::default();

            embed
                .title("bug report")
                .description("react to a message with 🐞 to generate one of these reports!

                report context:")
                .thumbnail("https://em-content.zobj.net/source/twitter/376/lady-beetle_1f41e.png")
                .color(Color::from_rgb(221, 46, 68))
                .fields(messages)
                .footer(|footer| { 
                    footer.icon_url("https://media.discordapp.net/attachments/1159320580823191672/1198721314802896996/9e1275360c072b9ad0c31d07d24f7257.webp?ex=65bfef38&is=65ad7a38&hm=2b4ab0cbd9b6497bc6e7cc96c4b537a197cef6fb30a22a4fc99432d8cd988aa0&=&format=webp&width=770&height=770")
                        .text("slimebot")
                })
                .timestamp(add_reaction.message_id.created_at());

            add_reaction
                .channel_id
                .send_message(ctx.http, |msg| msg.set_embed(embed.clone()))
                .await
                .unwrap();
        }
    }
}
