use poise::serenity_prelude::{
    self as serenity, CacheHttp, ComponentInteraction, CreateInteractionResponseMessage,
};

pub trait ComponentInteractionExt {
    async fn acknowledge(&self, cache_http: impl CacheHttp) -> serenity::Result<()>;
    async fn respond(
        &self,
        cache_http: impl CacheHttp,
        message: serenity::CreateInteractionResponseMessage,
    ) -> serenity::Result<()>;

    #[allow(dead_code)]
    async fn update_message(
        &self,
        cache_http: impl CacheHttp,
        builder: serenity::CreateInteractionResponseMessage,
    ) -> serenity::Result<()>;

    fn custom_id(&self) -> &str;

    #[allow(dead_code)]
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
    #[allow(dead_code)]
    fn is_some_with_id(&self, custom_id: &str) -> bool;
}

impl OptionComponentInteractionExt for Option<ComponentInteraction> {
    fn is_some_with_id(&self, custom_id: &str) -> bool {
        self.as_ref().is_some_and(|ci| ci.custom_id() == custom_id)
    }
}
