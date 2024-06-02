use poise::serenity_prelude::{self, CacheHttp, ChannelId, EditChannel, GuildChannel, Message};

use crate::discord::commands::SendMessageError;

trait RenameGuildChannel {
    async fn rename(
        &mut self,
        cache_http: impl CacheHttp,
        name: &str,
    ) -> serenity_prelude::Result<()>;
}

impl RenameGuildChannel for GuildChannel {
    async fn rename(
        &mut self,
        cache_http: impl CacheHttp,
        name: &str,
    ) -> serenity_prelude::Result<()> {
        self.edit(cache_http, EditChannel::new().name(name)).await
    }
}

trait RenameChannelId {
    async fn rename(
        self,
        cache_http: impl CacheHttp,
        name: &str,
    ) -> serenity_prelude::Result<GuildChannel>;
}

impl RenameChannelId for ChannelId {
    async fn rename(
        self,
        cache_http: impl CacheHttp,
        name: &str,
    ) -> serenity_prelude::Result<GuildChannel> {
        self.edit(cache_http, EditChannel::new().name(name)).await
    }
}

pub trait ChannelIdExt {
    async fn say_ext(
        self,
        cache_http: impl CacheHttp,
        content: impl Into<String>,
    ) -> Result<Message, SendMessageError>;
}

impl ChannelIdExt for ChannelId {
    async fn say_ext(
        self,
        cache_http: impl CacheHttp,
        content: impl Into<String>,
    ) -> Result<Message, SendMessageError> {
        self.say(cache_http, content)
            .await
            .map_err(SendMessageError::from)
    }
}
