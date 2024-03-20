use poise::{
    serenity_prelude::{self as serenity, CacheHttp, CreateActionRow, CreateButton},
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
}
