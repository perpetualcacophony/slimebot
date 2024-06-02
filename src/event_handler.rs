use std::{fmt, future::Future, pin::Pin};

use poise::{
    serenity_prelude::{self as serenity, CacheHttp, FullEvent, Message},
    FrameworkContext,
};
use thiserror::Error;
use tracing::trace;

use crate::{errors::CommandError, Data};

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    MessageWatchers(#[from] MessageWatchersErrors),
}

#[derive(Debug)]
struct MessageWatchersErrors {
    failures: Vec<CommandError>,
}

impl fmt::Display for MessageWatchersErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for err in &self.failures {
            write!(f, "{err}")?;
        }

        Ok(())
    }
}

impl std::error::Error for MessageWatchersErrors {}

// this is an awful use of TryFrom
impl TryFrom<[Result<(), CommandError>; 4]> for MessageWatchersErrors {
    type Error = ();

    fn try_from(value: [Result<(), CommandError>; 4]) -> Result<Self, Self::Error> {
        let mut failures = Vec::with_capacity(4);

        for result in value {
            if let Err(err) = result {
                failures.push(err)
            }
        }

        if failures.is_empty() {
            Err(())
        } else {
            Ok(Self { failures })
        }
    }
}

impl
    TryFrom<(
        Result<(), CommandError>,
        Result<(), CommandError>,
        Result<(), CommandError>,
        Result<(), CommandError>,
    )> for MessageWatchersErrors
{
    type Error = ();

    fn try_from(
        value: (
            Result<(), CommandError>,
            Result<(), CommandError>,
            Result<(), CommandError>,
            Result<(), CommandError>,
        ),
    ) -> Result<Self, Self::Error> {
        let array: [Result<(), CommandError>; 4] = value.into();
        array.try_into()
    }
}

async fn event_handler(
    serenity_ctx: &serenity::Context,
    event: &FullEvent,
    framework_ctx: FrameworkContext<'_, Data, CommandError>,
    data: &Data,
) -> Result<(), Error> {
    let filter_watcher_msg = move |msg: &Message| {
        !msg.is_own(&serenity_ctx.cache)
            && !msg.is_private()
            && data.config().watchers.channel_allowed(msg.channel_id)
    };

    match event {
        FullEvent::Message { new_message: msg } if filter_watcher_msg(msg) => {
            use crate::discord::watchers::*;

            let http = serenity_ctx.http();

            let results: Result<MessageWatchersErrors, ()> = tokio::join!(
                vore(http, &data.db, msg),
                l_biden(http, msg),
                look_cl(http, msg),
                watch_haiku(http, msg),
            )
            .try_into();

            // have to flip these because the TryFrom impl is backwards
            let results = match results {
                Ok(err) => Err(err),
                Err(_) => Ok(()),
            };

            results?;
        }
        FullEvent::ReactionAdd {
            add_reaction: reaction,
        } if reaction.user_id != Some(framework_ctx.bot_id) && reaction.guild_id.is_some() => {
            trace!(?reaction.message_id, "reaction captured");
            use crate::discord::bug_reports::bug_reports;

            if let Some(channel) = data.config().bug_reports_channel() {
                bug_reports(serenity_ctx.http(), reaction.clone(), channel).await;
            }
        }
        _ => (),
    }

    Ok(())
}

pub fn poise<'a>(
    serenity_ctx: &'a serenity::Context,
    event: &'a FullEvent,
    framework_ctx: FrameworkContext<'a, Data, CommandError>,
    data: &'a Data,
) -> Pin<Box<dyn Future<Output = Result<(), CommandError>> + Send + 'a>> {
    Box::pin(async move {
        event_handler(serenity_ctx, event, framework_ctx, data)
            .await
            .map_err(CommandError::from)
    })
}
