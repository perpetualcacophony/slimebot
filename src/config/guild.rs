use chrono::format;
use mongodb::{
    bson::{self, doc},
    Collection, Database,
};
use poise::serenity_prelude::{self as serenity, ChannelId, GuildId};
use serde::{de::Visitor, Deserialize};

type DbResult<T> = Result<T, mongodb::error::Error>;

#[derive(Deserialize)]
pub struct GuildConfig {
    guild_id: GuildId,
    prefix: Option<String>,
    watchers: WatchersConfig,
}

impl GuildConfig {
    async fn from_id(ctx: Context<'_>, id: GuildId) -> DbResult<Option<Self>> {
        let id = bson::ser::to_bson(&id).expect("GuildId implements Deserialize");

        ctx.collection.find_one(doc! { "guild_id": id }, None).await
    }
}

#[derive(Deserialize)]
pub struct WatchersConfig {
    enabled_by_default: bool,
    channel_overrides: Vec<(ChannelId, bool)>,
}

#[derive(Copy, Clone)]
pub struct Context<'a> {
    collection: &'a Collection<GuildConfig>,
}

impl Context<'_> {
    async fn get_collection(db: Database) -> Collection<GuildConfig> {
        db.collection(name)
    }

    async fn from_id(db: Database, id: GuildId) -> Self {
        let collection = db.collection(&format!("slimebot_{id}"));

        Self {
            collection: &collection,
        }
    }
}

trait ContextExt {
    async fn guild_config(&self) -> DbResult<Option<GuildConfig>>;
}

impl ContextExt for crate::Context<'_> {
    async fn guild_config(&self) -> DbResult<Option<GuildConfig>> {
        if let Some(guild_id) = self.guild_id() {
            GuildConfig::from_id(self, guild_id).await
        } else {
            Ok(None)
        }
    }
}
