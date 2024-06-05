use std::error::Error as _;

use poise::{serenity_prelude as serenity, BoxFuture, Context, FrameworkError};
use thiserror::Error as ThisError;
use tracing::{error, error_span, warn, Instrument};
use tracing_unwrap::ResultExt;

use crate::{framework::event_handler, PoiseData};

pub fn handle_framework_error(err: FrameworkError<'_, PoiseData, Error>) -> BoxFuture<()> {
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
async fn handle_error(err: Error, _ctx: Context<'_, PoiseData, Error>) {
    match err {
        Error::Command(cmd) => match cmd {
            CommandError::SendMessage(err) => error!("{err}"),
            CommandError::DiceRoll(err) => warn!("{err}"),
            other => error!("{}", other.source().expect("all variants have a source")),
        },
    }
}

#[derive(Debug, ThisError)]
pub enum CommandError {
    #[error("input error: {0}")]
    SendMessage(#[from] SendMessageError),
    #[error("other serenity error: {0}")]
    Serenity(#[from] serenity::Error),
    #[error("other reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("")]
    DiceRoll(#[from] DiceRollError),
    #[error("error from mongodb: {0}")]
    MongoDb(#[from] mongodb::error::Error),
    #[error("error from event handler: {0}")]
    EventHandler(#[from] event_handler::Error),
}

#[derive(Debug, ThisError)]
pub enum Error {
    #[error(transparent)]
    Command(#[from] CommandError),
}

#[derive(Debug, thiserror::Error)]
#[error("message failed to send")]
pub struct SendMessageError {
    #[from]
    pub source: serenity::Error,
}

impl SendMessageError {
    pub fn new(source: serenity::Error) -> Self {
        Self { source }
    }

    pub fn backoff(self) -> backoff::Error<Self> {
        use serenity::Error as E;

        match self.source {
            E::Model(ref model) => {
                use serenity::ModelError as M;

                match model {
                    M::InvalidPermissions { .. } | M::MessageTooLong(..) => {
                        backoff::Error::permanent(self)
                    }
                    _ => backoff::Error::transient(self),
                }
            }
            _ => backoff::Error::transient(self),
        }
    }
}

impl From<SendMessageError> for serenity::Error {
    fn from(val: SendMessageError) -> Self {
        val.source
    }
}

#[derive(Debug, ThisError, PartialEq)]
pub enum DiceRollError {
    #[error("")]
    NoFaces,
    #[error("")]
    InvalidExtra(String),
    #[error("'{0}' is not a valid sign, expected '+' or '-'")]
    InvalidExtraSign(String),
    #[error("no match in `{0}`")]
    NoMatch(String),
}
