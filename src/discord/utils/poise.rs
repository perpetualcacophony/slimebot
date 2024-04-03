use poise::{serenity_prelude as serenity, CreateReply};

pub trait ContextExt {
    async fn reply_ephemeral(
        &self,
        text: impl Into<String>,
    ) -> Result<poise::ReplyHandle<'_>, serenity::Error>;
}

impl ContextExt for crate::Context<'_> {
    async fn reply_ephemeral(
        &self,
        text: impl Into<String>,
    ) -> Result<poise::ReplyHandle<'_>, serenity::Error> {
        let builder = CreateReply::new().reply(true).ephemeral(true).content(text);
        self.send(builder).await
    }
}

pub trait CreateReplyExt: Default {
    fn new() -> Self {
        Self::default()
    }
}

impl CreateReplyExt for CreateReply {}
