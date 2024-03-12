use poise::{serenity_prelude as serenity, BoxFuture, Context, FrameworkError};
use thiserror::Error;
use tracing::{error, error_span, warn, Instrument};
use tracing_unwrap::ResultExt;

use crate::{roll::DiceRollError, Data};

#[derive(Debug, Error)]
pub enum Error {
    #[error("user error: {0}")]
    User(#[from] UserError),
    #[error("bot error: {0}")]
    Bot(#[from] BotError),
    #[error("manual error: {0:#}")]
    Manual(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum BotError {
    #[error(transparent)]
    Serenity(#[from] serenity::Error),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
}

#[derive(Debug, Error)]
pub enum UserError {
    #[error("invalid input: {0}")]
    Input(InputError),
}

#[derive(Debug, Error)]
pub enum InputError {
    #[error(transparent)]
    DiceRoll(#[from] DiceRollError),
}

impl From<serenity::Error> for Error {
    fn from(value: serenity::Error) -> Self {
        Self::Bot(BotError::from(value))
    }
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Self::Bot(BotError::from(value))
    }
}

impl From<DiceRollError> for Error {
    fn from(value: DiceRollError) -> Self {
        Self::User(UserError::Input(InputError::from(value)))
    }
}

pub fn handle_framework_error(err: FrameworkError<'_, Data, Error>) -> BoxFuture<()> {
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

async fn handle_error(err: Error, ctx: Context<'_, Data, Error>) {
    match err {
        Error::User(u) => {
            warn!(%u);
            ctx.reply(u.to_string())
                .await
                .expect_or_log("failed to send discord error message");
        }
        Error::Bot(_) => {
            error!("{err}");
        }
        _ => {
            error!("{err}");
        }
    }
}
