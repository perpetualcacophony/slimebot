use poise::{
    serenity_prelude::{self as serenity, Permissions},
    BoxFuture, Context, FrameworkError,
};

use thiserror::Error as ThisError;
use thisslime::TracingError;
//use tokio::sync::mpsc;
use tracing::{error, error_span, Instrument};
use tracing_unwrap::ResultExt;

use crate::{
    framework::event_handler::{self, HandlerError},
    utils::poise::ContextExt,
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
async fn handle_error(mut err: Error, ctx: Context<'_, PoiseData, Error>) {
    if let Error::Command(CommandError::Minecraft(
        crate::commands::minecraft::Error::AlreadyClaimed(ref mut err),
    )) = err
    {
        err.update_user_nick(ctx, ctx.guild_id())
            .await
            .expect("updating error should not fail");

        /*         let embed = err.create_embed(ctx);
        ctx.send_ext(poise::CreateReply::default().embed(embed))
            .await; */
    }

    err.trace();
    ctx.reply_ext(err.to_string())
        .await
        .expect("sending error message should not fail");
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
    Minecraft(#[from] crate::commands::minecraft::Error),

    #[cfg(feature = "nortverse")]
    #[error("error from nortverse api: {0}")]
    Nortverse(#[from] crate::commands::nortverse::Error),

    #[cfg(feature = "dynasty")]
    #[error("error from dynasty scans api: {0}")]
    #[event(level = ERROR)]
    Dynasty(#[from] dynasty2::Error),
}

#[derive(Debug, ThisError, TracingError)]
#[span]
pub enum Error {
    #[error(transparent)]
    Command(#[from] CommandError),

    #[error(transparent)]
    Handler(#[from] HandlerError),

    #[error(transparent)]
    Data(#[from] crate::framework::DataError),

    #[error(transparent)]
    Config(#[from] crate::framework::config::Error),
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

/* impl<T: TracingError> TracingError for &T {
    const LEVEL: tracing::Level = T::LEVEL;

    fn event(&self) {
        T::event(self)
    }
} */

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

/* #[derive(Clone, Debug)]
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
} */

/* impl<T: TracingError + std::error::Error> ErrorEmbed for T {
    default fn color(&self) -> serenity::Color {
        match self.level() {
            tracing::Level::ERROR => serenity::Color::RED,
            tracing::Level::WARN => serenity::Color::GOLD,
            _ => serenity::Color::FOOYOO,
        }
    }
} */

pub trait ErrorEmbed: std::fmt::Display {
    fn create_embed(&self, ctx: Context<'_, PoiseData, Error>) -> serenity::CreateEmbed;

    /*     async fn send_embed(
        &self,
        ctx: Context<'_, PoiseData, Error>,
    ) -> serenity::Result<serenity::Message> {
        let embed = self.create_embed(ctx);

        let message = ctx
            .send_ext(poise::CreateReply::default().reply(true).embed(embed))
            .await?
            .into_message()
            .await?;

        Ok(message)
    } */
}

pub trait ErrorEmbedOptions: std::fmt::Display {
    fn color(&self) -> serenity::Color;

    fn footer_text(&self) -> Option<impl Into<String>> {
        None::<String>
    }

    fn footer_icon_url(&self) -> Option<impl Into<String>> {
        None::<String>
    }

    fn title(&self) -> impl Into<String> {
        "boop"
    }

    fn description(&self) -> impl Into<String> {
        self.to_string()
    }
}

impl<T: ErrorEmbedOptions> ErrorEmbed for T {
    default fn create_embed(&self, ctx: Context<'_, PoiseData, Error>) -> serenity::CreateEmbed {
        let mut embed = serenity::CreateEmbed::new()
            .color(self.color())
            .description(self.description())
            .title(self.title());

        let footer = match (self.footer_text(), self.footer_icon_url()) {
            (Some(text), Some(icon)) => Some(serenity::CreateEmbedFooter::new(text).icon_url(icon)),
            (Some(text), None) => Some(serenity::CreateEmbedFooter::new(text)),
            _ => None,
        };

        if let Some(footer) = footer {
            embed = embed.footer(footer)
        }

        embed
    }
}
