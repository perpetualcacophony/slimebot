use crate::{discord::commands::SendMessageError, Data};

use poise::CreateReply;

pub type Context<'a> = poise::Context<'a, Data, crate::errors::CommandError>;

pub type Error = crate::errors::CommandError;
pub type Command = poise::Command<Data, Error>;
pub type CommandResult = Result<(), Error>;

pub trait ContextExt<'a> {
    async fn reply_ephemeral(
        self,
        text: impl Into<String>,
    ) -> Result<poise::ReplyHandle<'a>, SendMessageError>;

    fn in_guild(&self) -> bool;

    #[allow(dead_code)]
    fn in_dm(&self) -> bool {
        !self.in_guild()
    }

    async fn reply_ext(
        self,
        text: impl Into<String>,
    ) -> Result<poise::ReplyHandle<'a>, SendMessageError>;

    async fn send_ext(
        self,
        builder: CreateReply,
    ) -> Result<poise::ReplyHandle<'a>, SendMessageError>;

    async fn say_ext(
        self,
        text: impl Into<String>,
    ) -> Result<poise::ReplyHandle<'a>, SendMessageError>;
}

impl<'a> ContextExt<'a> for Context<'a> {
    async fn reply_ephemeral(
        self,
        text: impl Into<String>,
    ) -> Result<poise::ReplyHandle<'a>, SendMessageError> {
        let builder = CreateReply::new().reply(true).ephemeral(true).content(text);
        self.send_ext(builder).await
    }

    fn in_guild(&self) -> bool {
        self.guild_id().is_some()
    }

    async fn reply_ext(
        self,
        text: impl Into<String>,
    ) -> Result<poise::ReplyHandle<'a>, SendMessageError> {
        Self::reply(self, text).await.map_err(SendMessageError::new)
    }

    async fn send_ext(
        self,
        builder: CreateReply,
    ) -> Result<poise::ReplyHandle<'a>, SendMessageError> {
        Self::send(self, builder)
            .await
            .map_err(SendMessageError::new)
    }

    async fn say_ext(
        self,
        text: impl Into<String>,
    ) -> Result<poise::ReplyHandle<'a>, SendMessageError> {
        self.say(text).await.map_err(SendMessageError::from)
    }
}

pub trait CreateReplyExt {
    #[allow(dead_code)]
    fn new() -> Self
    where
        Self: Default,
    {
        Self::default()
    }
}

impl CreateReplyExt for CreateReply {}
