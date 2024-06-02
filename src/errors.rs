use std::error::Error;

use poise::{serenity_prelude as serenity, BoxFuture, Context, FrameworkError};
use thiserror::Error;
use tracing::{error, error_span, warn, Instrument};
use tracing_unwrap::ResultExt;

use crate::{event_handler, functions::misc::roll::DiceRollError, Data};

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
async fn handle_error(err: CommandError, _ctx: Context<'_, Data, CommandError>) {
    match err {
        CommandError::SendMessage(err) => error!("{err}"),
        CommandError::DiceRoll(err) => warn!("{err}"),
        other => error!("{}", other.source().expect("all variants have a source")),
    }
}

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
    #[error("error from event handler: {0}")]
    EventHandler(#[from] event_handler::Error),
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
