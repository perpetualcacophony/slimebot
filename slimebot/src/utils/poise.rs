use crate::{
    errors::{CommandError, Error, SendMessageError},
    PoiseData,
};

use poise::CreateReply;
use tracing::warn;

pub type Context<'a> = poise::Context<'a, PoiseData, crate::errors::Error>;

pub type Command = poise::Command<PoiseData, Error>;
pub type CommandResult = Result<(), CommandError>;

pub trait ContextExt<'a>: Into<Context<'a>> + Copy {
    async fn reply_ephemeral(
        self,
        text: impl Into<String>,
    ) -> Result<poise::ReplyHandle<'a>, SendMessageError>;

    fn in_guild(self) -> bool;

    #[allow(dead_code)]
    fn in_dm(self) -> bool {
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

    fn in_guild(self) -> bool {
        self.guild_id().is_some()
    }

    async fn reply_ext(
        self,
        text: impl Into<String>,
    ) -> Result<poise::ReplyHandle<'a>, SendMessageError> {
        self.send_ext(CreateReply::default().reply(true).content(text))
            .await
    }

    async fn send_ext(
        self,
        builder: CreateReply,
    ) -> Result<poise::ReplyHandle<'a>, SendMessageError> {
        backoff::future::retry_notify(
            backoff::ExponentialBackoff::default(),
            || async {
                self.send(builder.clone())
                    .await
                    .map_err(SendMessageError::from)
                    .map_err(SendMessageError::backoff)
            },
            |err, _| warn!("{err}, retrying..."),
        )
        .await
    }

    async fn say_ext(
        self,
        text: impl Into<String>,
    ) -> Result<poise::ReplyHandle<'a>, SendMessageError> {
        self.send_ext(CreateReply::default().content(text)).await
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
