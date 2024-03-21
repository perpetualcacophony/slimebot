use poise::{
    serenity_prelude::{
        self as serenity, CacheHttp, ComponentInteraction, CreateActionRow, CreateButton,
        CreateInteractionResponseMessage,
    },
    CreateReply,
};

pub trait CreateReplyExt: Default {
    fn new() -> Self {
        Self::default()
    }

    fn button(self, button: CreateButton) -> Self;
}

impl CreateReplyExt for CreateReply {
    fn button(mut self, button: CreateButton) -> Self {
        if let Some(ref mut rows) = self.components {
            if let Some(buttons) = rows.iter_mut().find_map(|row| match row {
                CreateActionRow::Buttons(b) => Some(b),
                _ => None,
            }) {
                buttons.push(button);
            } else {
                rows.push(CreateActionRow::Buttons(vec![button]));
            }
        } else {
            self = self.components(vec![CreateActionRow::Buttons(vec![button])]);
        }

        self
    }
}

pub trait ComponentInteractionExt {
    async fn acknowledge(&self, cache_http: impl CacheHttp) -> serenity::Result<()>;
    async fn respond(
        &self,
        cache_http: impl CacheHttp,
        message: serenity::CreateInteractionResponseMessage,
    ) -> serenity::Result<()>;
    async fn update_message(
        &self,
        cache_http: impl CacheHttp,
        builder: serenity::CreateInteractionResponseMessage,
    ) -> serenity::Result<()>;
    fn custom_id(&self) -> &str;

    async fn reply(
        &self,
        cache_http: impl CacheHttp,
        content: impl Into<String>,
    ) -> serenity::Result<()> {
        self.respond(
            cache_http,
            CreateInteractionResponseMessage::new().content(content),
        )
        .await
    }
    async fn reply_ephemeral(
        &self,
        cache_http: impl CacheHttp,
        content: impl Into<String>,
    ) -> serenity::Result<()> {
        self.respond(
            cache_http,
            CreateInteractionResponseMessage::new()
                .content(content)
                .ephemeral(true),
        )
        .await
    }
}

impl ComponentInteractionExt for serenity::ComponentInteraction {
    async fn acknowledge(&self, cache_http: impl CacheHttp) -> serenity::Result<()> {
        let builder = serenity::CreateInteractionResponse::Acknowledge;
        self.create_response(cache_http, builder).await
    }

    async fn respond(
        &self,
        cache_http: impl CacheHttp,
        message: serenity::CreateInteractionResponseMessage,
    ) -> serenity::Result<()> {
        let builder = serenity::CreateInteractionResponse::Message(message);
        self.create_response(cache_http, builder).await
    }

    async fn update_message(
        &self,
        cache_http: impl CacheHttp,
        builder: serenity::CreateInteractionResponseMessage,
    ) -> serenity::Result<()> {
        let builder = serenity::CreateInteractionResponse::UpdateMessage(builder);
        self.create_response(cache_http, builder).await
    }

    fn custom_id(&self) -> &str {
        &self.data.custom_id
    }
}

pub trait OptionComponentInteractionExt {
    fn is_some_with_id(&self, custom_id: &str) -> bool;
}

impl OptionComponentInteractionExt for Option<ComponentInteraction> {
    fn is_some_with_id(&self, custom_id: &str) -> bool {
        self.as_ref().is_some_and(|ci| ci.custom_id() == custom_id)
    }
}
