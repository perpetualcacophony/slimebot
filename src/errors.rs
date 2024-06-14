use poise::{
    serenity_prelude::{self as serenity, Permissions},
    BoxFuture, Context, FrameworkError,
};
use slimebot_macros::TracingError;
use thiserror::Error as ThisError;
use tokio::sync::mpsc;
use tracing::{error, error_span, Instrument};
use tracing_unwrap::ResultExt;

use crate::{
    framework::event_handler::{self, HandlerError},
    PoiseData,
};

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
    err.event();
}

#[derive(Debug, ThisError, TracingError)]
#[span(level = WARN)]
pub enum CommandError {
    #[error("input error: {0}")]
    SendMessage(#[from] SendMessageError),

    #[error("other serenity error: {0}")]
    #[event(level = ERROR)]
    Serenity(#[from] serenity::Error),

    #[error("other reqwest error: {0}")]
    #[event(level = ERROR)]
    Reqwest(#[from] reqwest::Error),

    #[error("")]
    #[event(level = ERROR)]
    DiceRoll(#[from] DiceRollError),

    #[error("error from mongodb: {0}")]
    #[event(level = ERROR)]
    MongoDb(#[from] mongodb::error::Error),

    #[error("error from event handler: {0}")]
    EventHandler(#[from] event_handler::HandlerError),

    #[error("error from minecraft api: {0}")]
    MinecraftApi(#[from] crate::commands::minecraft::api::Error),
}

#[derive(Debug, ThisError, TracingError)]
#[span]
pub enum Error {
    #[error(transparent)]
    Command(#[from] CommandError),

    #[error(transparent)]
    Handler(#[from] HandlerError),
}

#[derive(Debug, thiserror::Error, TracingError)]
#[span(level = ERROR)]
pub enum SendMessageError {
    #[error(transparent)]
    Permissions(#[from] MissingPermissionsError),

    #[error(transparent)]
    MessageTooLong(#[from] MessageTooLongError),

    #[error("boop")]
    #[event(level = ERROR)]
    Other(serenity::Error),
}

impl From<serenity::Error> for SendMessageError {
    fn from(value: serenity::Error) -> Self {
        match value {
            serenity::Error::Model(ref model) => match model {
                serenity::ModelError::InvalidPermissions { required, present } => {
                    Self::Permissions(MissingPermissionsError {
                        required: *required,
                        present: *present,
                    })
                }
                serenity::ModelError::MessageTooLong(len) => {
                    Self::MessageTooLong(MessageTooLongError { length: *len })
                }
                _ => Self::Other(value),
            },
            _ => Self::Other(value),
        }
    }
}

#[derive(Debug, ThisError, TracingError)]
#[error("missing permissions: {}", self.missing())]
#[event(level = ERROR)]
pub struct MissingPermissionsError {
    #[field(print = Display)]
    required: Permissions,

    #[field(print = Display)]
    present: Permissions,
}

impl MissingPermissionsError {
    fn missing(&self) -> Permissions {
        self.required.difference(self.present)
    }
}

#[derive(Debug, ThisError, TracingError)]
#[event(level = ERROR)]
#[error("message is too long")]
pub struct MessageTooLongError {
    pub length: usize,
}

impl SendMessageError {
    pub fn backoff(self) -> backoff::Error<Self> {
        match self {
            Self::MessageTooLong(_) | Self::Permissions(_) => backoff::Error::permanent(self),
            _ => backoff::Error::transient(self),
        }
    }
}

impl From<SendMessageError> for serenity::Error {
    fn from(val: SendMessageError) -> Self {
        match val {
            SendMessageError::MessageTooLong(err) => {
                Self::Model(serenity::ModelError::MessageTooLong(err.length))
            }
            SendMessageError::Permissions(err) => {
                Self::Model(serenity::ModelError::InvalidPermissions {
                    required: err.required,
                    present: err.present,
                })
            }
            SendMessageError::Other(source) => source,
        }
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

pub trait TracingError: std::error::Error {
    fn event(&self);
}

/*impl TracingError for SendMessageError {
    fn event(&self) {
        match self {
            Self::MessageTooLong(err) => {
                error_fields!(tracing::Level::ERROR; err => MessageTooLongError[length]);
            }
            Self::Permissions(err) => {
                error_fields!(tracing::Level::ERROR; err => MissingPermissionsError[required% present%])
            }
            other => error!("{}", other),
        }
    }
}*/

#[derive(Clone, Debug)]
pub struct ErrorSender {
    tx: mpsc::Sender<FrameworkError<'static, PoiseData, Error>>,
}

impl ErrorSender {
    fn new(tx: mpsc::Sender<FrameworkError<'static, PoiseData, Error>>) -> Self {
        Self { tx }
    }

    pub async fn send(&self, err: FrameworkError<'static, PoiseData, Error>) {
        self.tx.send(err).await;
    }
}

pub struct ErrorHandler {
    rx: mpsc::Receiver<FrameworkError<'static, PoiseData, Error>>,
}

impl ErrorHandler {
    fn new(rx: mpsc::Receiver<FrameworkError<'static, PoiseData, Error>>) -> Self {
        Self { rx }
    }

    pub fn channel() -> (ErrorSender, ErrorHandler) {
        let (tx, rx) = mpsc::channel(10);
        (ErrorSender::new(tx), ErrorHandler::new(rx))
    }

    async fn recv(&mut self) -> Option<FrameworkError<'static, PoiseData, Error>> {
        self.rx.recv().await
    }

    pub fn spawn(mut self) {
        tokio::spawn(async move {
            while let Some(err) = self.recv().await {
                todo!("handle the error")
            }
        });
    }
}
