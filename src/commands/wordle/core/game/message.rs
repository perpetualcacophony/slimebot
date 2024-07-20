use poise::serenity_prelude::{
    self, futures::Stream, CacheHttp, ChannelId, ComponentInteraction, CreateActionRow,
    CreateButton, CreateMessage, EditMessage, Message, MessageId, ReactionType, Result,
    ShardMessenger,
};

use crate::utils::{poise::ContextExt, Context};

use super::{options::GameStyle, GameContext, GameData};

use super::super::{core::AsEmoji, Puzzle};

pub struct GameMessage {
    msg: Message,
    style: GameStyle,
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

    fn content(data: &impl AsRef<GameData>, style: GameStyle) -> String {
        let data = data.as_ref();

        format!(
            "{title} {guesses}/{max}\n{emojis}",
            title = data.puzzle.title(),
            guesses = data.guesses.count(),
            max = data
                .guesses
                .max()
                .map_or("∞".to_owned(), |lim| lim.to_string()),
            emojis = data.guesses.emoji_with_style(style)
        )
    }

    fn builder(data: impl AsRef<GameData>, style: GameStyle) -> EditMessage {
        EditMessage::new()
            .content(Self::content(&data, style))
            .components(Self::buttons(&data))
    }

    pub async fn new(ctx: Context<'_>, puzzle: &Puzzle, style: GameStyle) -> Result<Self> {
        Ok(Self {
            msg: Self::loading_msg(ctx, puzzle).await?,
            style,
        })
    }

    pub async fn loading_msg(ctx: Context<'_>, puzzle: &Puzzle) -> Result<Message> {
        let msg = if puzzle.is_daily() && ctx.in_guild() {
            ctx.reply_ephemeral("you can't play a daily wordle in a server - check your dms!")
                .await
                .map_err(serenity_prelude::Error::from)?;

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
        let builder = Self::builder(data, self.style);
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
