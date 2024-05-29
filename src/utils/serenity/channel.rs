use poise::serenity_prelude::{self, CacheHttp, ChannelId, EditChannel, GuildChannel};

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
