use poise::serenity_prelude as serenity;

use crate::errors::TracingError;

#[derive(Debug, Clone, thiserror::Error, slimebot_macros::TracingError)]
#[event(level = WARN)]
pub struct ErrorAlreadyClaimed {
    #[field(print = Display)]
    pub(crate) user_id: serenity::UserId,
    pub(crate) user_name: Option<String>,
    pub(crate) minecraft_username: String,
}

impl ErrorAlreadyClaimed {
    pub(crate) fn new(
        user_id: serenity::UserId,
        user_name: Option<String>,
        minecraft_username: String,
    ) -> Self {
        Self {
            user_id,
            user_name,
            minecraft_username,
        }
    }

    pub(crate) fn from_serenity(user: &serenity::User, minecraft_username: String) -> Self {
        Self::new(user.id, Some(user.name.clone()), minecraft_username)
    }

    pub(crate) fn set_user_name(&mut self, new: String) {
        self.user_name = Some(new)
    }

    pub async fn update_user_name(
        &mut self,
        cache_http: impl serenity::CacheHttp,
    ) -> serenity::Result<()> {
        let user = self.user_id.to_user(cache_http).await?;
        self.set_user_name(user.name);
        Ok(())
    }

    pub async fn update_user_nick(
        &mut self,
        cache_http: impl serenity::CacheHttp,
        guild_id: Option<serenity::GuildId>,
    ) -> serenity::Result<()> {
        let user = self.user_id.to_user(&cache_http).await?;
        let nick = if let Some(guild_id) = guild_id {
            user.nick_in(&cache_http, guild_id)
                .await
                .unwrap_or(user.name)
        } else {
            user.name
        };
        self.set_user_name(nick);
        Ok(())
    }
}

impl std::fmt::Display for ErrorAlreadyClaimed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let already_claimed_by = if let Some(ref name) = self.user_name {
            format!("{name} ({id})", id = self.user_id)
        } else {
            format!("user {id}", id = self.user_id)
        };

        write!(
            f,
            "minecraft user {username} already claimed by {already_claimed_by}",
            username = self.minecraft_username
        )
    }
}

#[derive(Debug, thiserror::Error, slimebot_macros::TracingError)]
pub enum Error {
    #[error(transparent)]
    AlreadyClaimed(#[from] ErrorAlreadyClaimed),

    #[error(transparent)]
    Api(#[from] super::api::Error),

    #[error("error from mongodb: {0}")]
    #[event(level = ERROR)]
    MongoDb(#[from] mongodb::error::Error),
}
