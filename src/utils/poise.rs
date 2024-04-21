use crate::Data;

use poise::{serenity_prelude as serenity, CreateReply};

use super::serenity::buttons::AddButton;

pub type Context<'a> = poise::Context<'a, Data, crate::errors::CommandError>;

pub type Error = crate::errors::CommandError;
pub type Command = poise::Command<Data, Error>;
pub type CommandResult = Result<(), Error>;

pub trait ContextExt {
    async fn reply_ephemeral(
        &self,
        text: impl Into<String>,
    ) -> Result<poise::ReplyHandle<'_>, serenity::Error>;

    fn in_guild(&self) -> bool;

    fn in_dm(&self) -> bool {
        !self.in_guild()
    }
}

impl ContextExt for Context<'_> {
    async fn reply_ephemeral(
        &self,
        text: impl Into<String>,
    ) -> Result<poise::ReplyHandle<'_>, serenity::Error> {
        let builder = CreateReply::new().reply(true).ephemeral(true).content(text);
        self.send(builder).await
    }

    fn in_guild(&self) -> bool {
        self.guild_id().is_some()
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
