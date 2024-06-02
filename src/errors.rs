use std::error::Error;

use poise::{serenity_prelude as serenity, BoxFuture, Context, FrameworkError};
use thiserror::Error;
use tracing::{error, error_span, warn, Instrument};
use tracing_unwrap::ResultExt;

use crate::{functions::misc::roll::DiceRollError, Data};

/*#[derive(Debug, Error)]
pub enum Error {
    #[error("user error: {0}")]
    Input(#[from] InputError),
    #[error("bot error: {0}")]
    Bot(BotError),
}*/

#[derive(Debug, Error)]
pub enum BotError {
    #[error(transparent)]
    Serenity(#[from] serenity::Error),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error("mongodb error: {0:#}")]
    MongoDb(#[from] mongodb::error::Error),
}

#[derive(Debug, Error)]
pub enum InputError {
    #[error(transparent)]
    DiceRoll(#[from] DiceRollError),
    #[error("{0}")]
    Other(String),
}

/*use crate::functions::games::wordle;
impl From<wordle::Error> for Error {
    fn from(value: wordle::Error) -> Self {
        match value {
            wordle::Error::MongoDb(err) => Self::Bot(BotError::MongoDb(err)),
            _ => unimplemented!(),
        }
    }
}*/

pub fn handle_framework_error(err: FrameworkError<'_, Data, CommandError>) -> BoxFuture<()> {
    Box::pin(async {
        match err {
            FrameworkError::Command { error, ctx, .. } => {
                let command = ctx.invoked_command_name();
                let span = error_span!("", command);
                let _enter = span.enter();

                handle_error(error, ctx).in_current_span().await;
            }
            FrameworkError::MissingBotPermissions {
                missing_permissions,
                ctx,
                ..
            } => {
                let command = ctx.invoked_command_name();
                let span = error_span!("", command);
                let _enter = span.enter();

                error!(%missing_permissions, "bot is missing permissions");
            }
            _ => {
                poise::builtins::on_error(err)
                    .await
                    .expect_or_log("failed to handle framework error");
            }
        };
    })
}
async fn handle_error(err: CommandError, ctx: Context<'_, Data, CommandError>) {
    match err {
        CommandError::SendMessage(err) => error!("{err}"),
        CommandError::DiceRoll(err) => warn!("{err}"),
        other => error!("{}", other.source().expect("all variants have a source")),
    }
}

/*
        CommandError::Input(_) => {
            warn!(%err);
            ctx.reply(err.to_string())
                .await
                .expect_or_log("failed to send discord error message");
        }
        CommandError::Internal(_) => {
            error!("{err:#}");
        }
        _ => {
            error!("{err}");
        }
*/

#[derive(Debug, Error)]
pub enum CommandError {
    #[error("input error: {0}")]
    SendMessage(#[from] crate::discord::commands::SendMessageError),
    #[error("other serenity error: {0}")]
    Serenity(#[from] serenity::Error),
    #[error("other reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("")]
    DiceRoll(#[from] DiceRollError),
    #[error("error from mongodb: {0}")]
    MongoDb(#[from] mongodb::error::Error),
}

#[derive(Debug, Error)]
pub enum LibraryError {
    #[error("mongodb error")]
    MongoDb(#[from] mongodb::error::Error),
    #[error("reqwest error")]
    Reqwest(#[from] reqwest::Error),
    #[error("serenity error")]
    Serenity(#[from] serenity::Error),
}

/*impl From<mongodb::error::Error> for CommandError {
    fn from(value: mongodb::error::Error) -> Self {
        Self::Library(value.into())
    }
}

impl From<reqwest::Error> for CommandError {
    fn from(value: reqwest::Error) -> Self {
        Self::Library(value.into())
    }
}

impl From<serenity::Error> for CommandError {
    fn from(value: serenity::Error) -> Self {
        Self::Library(value.into())
    }
}*/

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("error from dog api (HTTP {0})")]
    DogCeo(u16),
}

#[derive(Debug, Error)]
pub enum InternalError {
    #[error(transparent)]
    Api(ApiError),
}
