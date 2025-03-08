use poise::serenity_prelude as serenity;

use crate::errors::SendMessageError;

pub trait AsDiscordId<Id: DiscordId> {
    type Id: DiscordId = Id;

    #[allow(unused)]
    fn as_id(&self) -> Id;
}

pub trait DiscordId: Copy + Into<u64> {}

macro_rules! auto_ids {
    ($(($t_first:ident$(::$first_method:ident())?$(.$first_field:ident)?$(, $t:ident$(::$method:ident())?$(.$field:ident)?)*))+) => {
        $(
            paste::paste! {
                use poise::serenity_prelude::{$t_first, [< $t_first Id >]};

                impl DiscordId for [< $t_first Id >] {}

                impl AsDiscordId<[< $t_first Id >]> for [< $t_first Id >] {
                    fn as_id(&self) -> Self::Id {
                        *self
                    }
                }

                impl<'a> AsDiscordId<[< $t_first Id >]> for &'a [< $t_first Id >] {
                    fn as_id(&self) -> Self::Id {
                        **self
                    }
                }

                impl AsDiscordId<[< $t_first Id >]> for $t_first {
                    fn as_id(&self) -> Self::Id {
                        self.$($first_method())?$($first_field)?
                    }
                }

                impl<'a> AsDiscordId<[< $t_first Id >]> for &'a $t_first {
                    fn as_id(&self) -> Self::Id {
                        self.$($first_method())?$($first_field)?
                    }
                }

                $(
                    use poise::serenity_prelude::$t;

                    impl AsDiscordId<[< $t_first Id >]> for $t {
                        fn as_id(&self) -> Self::Id {
                            self.$($method())?$($field)?
                        }
                    }

                    impl<'a> AsDiscordId<[< $t_first Id >]> for &'a $t {
                        fn as_id(&self) -> Self::Id {
                            self.$($method())?$($field)?
                        }
                    }
                )*
            }
        )+
    };
}

auto_ids! {
    (User.id)
    (Guild.id)
    (Channel::id(), GuildChannel.id, PrivateChannel.id)
}

pub trait UserIdExt {
    async fn dm_ext(
        self,
        cache_http: impl serenity::CacheHttp + Copy,
        builder: serenity::CreateMessage,
    ) -> Result<serenity::Message, SendMessageError>;
}

impl UserIdExt for serenity::UserId {
    async fn dm_ext(
        self,
        cache_http: impl serenity::CacheHttp + Copy,
        builder: serenity::CreateMessage,
    ) -> Result<serenity::Message, SendMessageError> {
        backoff::future::retry_notify(
            backoff::ExponentialBackoff::default(),
            || async {
                self.dm(cache_http, builder.clone())
                    .await
                    .map_err(SendMessageError::from)
                    .map_err(SendMessageError::backoff)
            },
            |err, _| tracing::warn!("{err}, retrying..."),
        )
        .await
    }
}
