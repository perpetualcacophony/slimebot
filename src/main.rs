#![warn(clippy::perf)]

/// Logging frontends, with [`tracing`](https://docs.rs/tracing/latest/tracing/) backend.
mod logging;
use std::pin::Pin;

use logging::DiscordSubscriber;

/// Functionality called from Discord.
mod discord;
#[allow(clippy::wildcard_imports)]
use discord::commands::*;
use discord::framework::Handler;
use mongodb::Database;

/// Config file parsing and option access.
mod config;

mod db;

use poise::{
    serenity_prelude::{self as serenity, futures::future, GatewayIntents, HttpError, ModelError, Permissions, SerenityError}, FrameworkError, PrefixFrameworkOptions
};
use tokio::join;
use tracing::{info, trace, error};
use tracing_unwrap::ResultExt;

use chrono::Utc;
type UtcDateTime = chrono::DateTime<Utc>;

#[derive(Debug)]
pub struct Data {
    config: config::Config,
    db: Database,
    started: UtcDateTime,
}

#[derive(thiserror::Error, Debug)]
enum BotError {
    #[error("model error")]
    Model(ModelError),
    #[error("permissions error")]
    Permissions(Permissions),
    #[error("serenity error {0}")]
    Serenity(#[from] SerenityError),
    #[error("http error")]
    Http(#[from] HttpError),
    #[error("anyhow error")]
    Anyhow(#[from] anyhow::Error),
    #[error("io error")]
    Io(#[from] std::io::Error),
}

impl From<ModelError> for BotError {
    fn from(value: ModelError) -> Self {
        match value {
            ModelError::InvalidPermissions(p) => Self::Permissions(p),
            e => Self::Model(e)
        }
    }
}

fn handle_error<'a>(error: FrameworkError<'_, Data, BotError>) -> std::pin::Pin<std::boxed::Box<(dyn std::future::Future<Output = ()> + std::marker::Send + 'a)>> {
    if let FrameworkError::Command { error, ctx: _ } = error {
        if let BotError::Permissions(p) = error {
            error!("invalid permissions: {p}")
        }
    } else {
        error!("unknown error")
    }

    Box::pin( dummy() )
}

async fn dummy() {}

impl Data {
    fn new() -> Self {
        let config: crate::config::Config = ::config::Config::builder()
            .add_source(::config::File::with_name("slimebot.toml"))
            .add_source(::config::Environment::with_prefix("SLIMEBOT"))
            .build()
            .expect_or_log("config file could not be loaded")
            .try_deserialize()
            .expect_or_log("configuration could not be parsed");

        trace!("config loaded");

        let db = db::database(&config.db);

        let started = Utc::now();

        Self {
            config,
            db,
            started,
        }
    }

    const fn config(&self) -> &crate::config::Config {
        &self.config
    }

    const fn db(&self) -> &Database {
        &self.db
    }
}

// i should replace this with anyhow::Error
//type Error = Box<dyn std::error::Error + Send + Sync>;

type DiscordToken = String;

#[tokio::main]
async fn main() {
    // the stdout logger is started, and returns the
    // receiver for initializing the discord logger later.
    // because that can't be done until we get the http from the framework
    let discord_receiver = DiscordSubscriber::init_stdout();

    let data = Data::new();
    let config = data.config.clone();

    if let Some(flavor_text) = config.logs.flavor_text() {
        info!("{flavor_text}")
    }

    let mut handler = Handler {
        data,
        options: poise::FrameworkOptions {
            commands: vec![
                ping(),
                pong(),
                pfp(),
                watch_fic(),
                echo(),
                ban(),
                banban(),
                uptime(),
            ],
            prefix_options: PrefixFrameworkOptions {
                prefix: Some(config.bot.prefix().to_string()),
                ..Default::default()
            },
            on_error: handle_error,
            ..Default::default()
        },
        shard_manager: std::sync::Mutex::new(None),
    };
    poise::set_qualified_names(&mut handler.options.commands);

    let handler = std::sync::Arc::new(handler);
    let mut client = serenity::Client::builder(config.bot.token(), GatewayIntents::all())
        .event_handler_arc(handler.clone())
        .await
        .unwrap();

    *handler.shard_manager.lock().unwrap() = Some(client.shard_manager.clone());

    /*
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![ping(), pfp(), watch_fic(), echo(), ban(), banban()],
            prefix_options: PrefixFrameworkOptions {
                prefix: Some("..".to_string()),
                ..Default::default()
            },
            ..Default::default()
        })
        .token(conf.bot.token())
        .intents(serenity::GatewayIntents::all())
        .client_settings(|client| client.register_songbird())
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_in_guild(
                    ctx,
                    &framework.options().commands,
                    GuildId(testing_server),
                )
                .await?;

                if let Some(status) = conf.bot.status {
                    let (kind, state) = status.split_once(" ").unwrap();
                    let activity = match kind {
                        "playing" => Activity::playing(state),
                        _ => {
                            error!("unknown activity \"{}\" in config", kind);
                            panic!()
                        }
                    };

                    ctx.set_activity(activity).await;
                }

                Ok(Data {})
            })
        })
        .build()
        .await
        .unwrap();
    */

    trace!("discord framework set up");

    // i don't like how far in you have to go to access this :<
    let http = client.cache_and_http.http.clone();

    if config.logs.discord.enabled() {
        DiscordSubscriber::init_discord(
            http.clone(),
            config.logs.discord.channel().unwrap().into(),
            discord_receiver,
        )
        .await;
        trace!("hi discord!");
    }

    trace!("discord framework started");
    client.start().await.unwrap();
}

trait FormatDuration {
    fn format_largest(&self) -> String;
    fn format_full(&self) -> String;
}

impl FormatDuration for chrono::Duration {
    fn format_largest(&self) -> String {
        let (d, h, m, s) = (
            self.num_days(),
            self.num_hours(),
            self.num_minutes(),
            self.num_seconds(),
        );

        match (d, h, m, s) {
            (1, _, _, _) => ("1 day").to_string(),
            (2.., _, _, _) => format!("{d} days"),
            (_, 1, _, _) => ("1 hour").to_string(),
            (_, 2.., _, _) => format!("{h} hours"),
            (_, _, 1, _) => ("1 minute").to_string(),
            (_, _, 2.., _) => format!("{m} minutes"),
            (_, _, _, 1) => ("1 second").to_string(),
            (_, _, _, 2..) => format!("{s} seconds"),
            (_, _, _, _) => "less than a second".to_string(),
        }
    }

    fn format_full(&self) -> String {
        let mut formatted = String::new();

        if self.num_days() > 0 {
            formatted += &format!("{}d ", self.num_days());
        }

        if self.num_hours() > 0 {
            formatted += &format!("{}h ", self.num_hours() - (self.num_days() * 24));
        }

        if self.num_minutes() > 0 {
            formatted += &format!("{}m", self.num_minutes() - (self.num_hours() * 60));
        } else {
            formatted = "less than a minute".to_string();
        }

        formatted
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::DateTime;
    use pretty_assertions::assert_eq;

    #[test]
    fn format_full() {
        let start = DateTime::parse_from_rfc3339("2024-01-19T20:00:00.000Z").unwrap();

        let end = DateTime::parse_from_rfc3339("2024-01-21T21:19:00.000Z").unwrap();

        let duration = end - start;

        assert_eq!("2d 1h 19m", duration.format_full(),)
    }

    #[test]
    fn format_largest() {
        let start = DateTime::parse_from_rfc3339("2024-01-19T20:00:00.000Z").unwrap();
        let end = DateTime::parse_from_rfc3339("2024-01-21T21:19:00.000Z").unwrap();
        let duration = end - start;
        assert_eq!("2 days", duration.format_largest(),);

        let start = DateTime::parse_from_rfc3339("2024-01-19T20:00:00.000Z").unwrap();
        let end = DateTime::parse_from_rfc3339("2024-01-19T21:19:00.000Z").unwrap();
        let duration = end - start;
        assert_eq!("1 hour", duration.format_largest(),);

        let start = DateTime::parse_from_rfc3339("2024-01-19T20:00:00.000Z").unwrap();
        let end = DateTime::parse_from_rfc3339("2024-01-19T20:19:00.000Z").unwrap();
        let duration = end - start;
        assert_eq!("19 minutes", duration.format_largest(),);
    }
}
