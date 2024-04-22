use std::{marker::PhantomData, sync::Arc};

use arc_swap::{ArcSwap, ArcSwapOption};
use poise::serenity_prelude::{
    futures::Stream, CacheHttp, ChannelId, ComponentInteraction, CreateActionRow, CreateButton,
    CreateMessage, EditMessage, GuildId, Message, MessageId, ReactionType, Result, ShardMessenger,
};

use crate::{
    functions::games::wordle::{
        core::{guess::GuessSlice, AsEmoji},
        Puzzle,
    },
    utils::{poise::ContextExt, Context},
};

use super::{GameContext, GameData};

enum NoMessage {}

pub struct GameMessage {
    msg: Message,
}

impl GameMessage {
    pub async fn reply(
        &self,
        cache_http: impl CacheHttp,
        content: impl Into<String>,
    ) -> Result<()> {
        self.msg.reply(cache_http, content).await?;
        Ok(())
    }

    pub async fn finish(
        &mut self,
        cache_http: impl CacheHttp,
        message: impl AsRef<str>,
    ) -> Result<()> {
        self.msg
            .edit(
                cache_http,
                EditMessage::new().content(self.msg.content.clone() + message.as_ref()),
            )
            .await
    }

    pub fn channel_id(&self) -> &ChannelId {
        &self.msg.channel_id
    }

    pub fn message_id(&self) -> &MessageId {
        &self.msg.id
    }

    pub fn replies_stream(&self, shard: impl AsRef<ShardMessenger>) -> impl Stream<Item = Message> {
        self.msg.channel_id.await_replies(shard).stream()
    }

    pub fn buttons_stream(
        &self,
        shard: impl AsRef<ShardMessenger>,
    ) -> impl Stream<Item = ComponentInteraction> {
        self.msg.await_component_interactions(shard).stream()
    }

    fn content(data: &impl AsRef<GameData>) -> String {
        let data = data.as_ref();

        format!(
            "{title} {guesses}/{max}\n{emojis}",
            title = data.puzzle.title(),
            guesses = data.guesses.count(),
            max = data
                .guesses
                .limit
                .map_or("∞".to_owned(), |lim| lim.to_string()),
            emojis = data.guesses.as_emoji()
        )
    }

    fn builder(data: impl AsRef<GameData>) -> EditMessage {
        EditMessage::new()
            .content(Self::content(&data))
            .components(Self::buttons(&data))
    }

    pub async fn new(ctx: Context<'_>, puzzle: &Puzzle) -> Result<Self> {
        Ok(Self {
            msg: Self::loading_msg(ctx, puzzle).await?,
        })
    }

    pub async fn loading_msg(ctx: Context<'_>, puzzle: &Puzzle) -> Result<Message> {
        let msg = if puzzle.is_daily() && ctx.in_guild() {
            ctx.reply_ephemeral("you can't play a daily wordle in a server - check your dms!")
                .await?;

            ctx.author()
                .dm(ctx, CreateMessage::new().content("loading..."))
                .await?
        } else {
            ctx.reply("loading...").await?.into_message().await?
        };

        Ok(msg)
    }

    pub async fn edit(
        &mut self,
        cache_http: impl CacheHttp,
        data: impl AsRef<GameData>,
    ) -> Result<()> {
        let builder = Self::builder(data);
        self.msg.edit(cache_http, builder).await?;

        Ok(())
    }

    pub async fn delete(&mut self, cache_http: impl CacheHttp) -> Result<()> {
        self.msg.delete(cache_http).await?;

        Ok(())
    }

    pub async fn resend(&mut self, ctx: GameContext<'_>) -> Result<()> {
        self.delete(ctx).await?;
        //self.reply_loading(ctx).await?;
        Ok(())
    }

    pub fn stop_buttons(data: impl AsRef<GameData>) -> CreateActionRow {
        let pause_cancel_button = if data.as_ref().puzzle.is_daily() {
            CreateButton::new("pause")
                .emoji(ReactionType::Unicode("⏸️".to_owned()))
                .label("pause")
                .style(poise::serenity_prelude::ButtonStyle::Primary)
        } else {
            CreateButton::new("cancel")
                .emoji(ReactionType::Unicode("🚫".to_owned()))
                .label("cancel")
                .style(poise::serenity_prelude::ButtonStyle::Secondary)
        };

        let give_up_button = CreateButton::new("give_up")
            .emoji(ReactionType::Unicode("🏳️".to_owned()))
            .label("give up")
            .style(poise::serenity_prelude::ButtonStyle::Danger);

        let buttons = vec![pause_cancel_button, give_up_button];

        CreateActionRow::Buttons(buttons)
    }

    pub fn info_buttons() -> CreateActionRow {
        let unused = CreateButton::new("unused")
            .emoji(ReactionType::Unicode("🔎".to_owned()))
            .label("unused letters");

        CreateActionRow::Buttons(vec![unused])
    }

    pub fn buttons(data: impl AsRef<GameData>) -> Vec<CreateActionRow> {
        vec![Self::stop_buttons(data), Self::info_buttons()]
    }
}
