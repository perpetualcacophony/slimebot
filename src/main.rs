#![warn(
    clippy::complexity,
    clippy::correctness,
    clippy::nursery,
    clippy::perf,
    clippy::restriction,
    clippy::style,
    clippy::suspicious
)]
#![allow(
    clippy::blanket_clippy_restriction_lints,
    clippy::missing_docs_in_private_items,
    clippy::single_call_fn,
    clippy::implicit_return
)]

/// Logging frontends, with [`tracing`](https://docs.rs/tracing/latest/tracing/) backend.
mod logging;
use std::{
    fmt::Write,
    sync::{Arc, Mutex},
};

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
    serenity_prelude::{self as serenity, GatewayIntents},
    PrefixFrameworkOptions,
};
use tracing::{info, trace};
use tracing_unwrap::ResultExt;

use chrono::Utc;
type UtcDateTime = chrono::DateTime<Utc>;

#[derive(Debug)]
pub struct Data {
    config: config::Config,
    db: Database,
    started: UtcDateTime,
}

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

    #[allow(clippy::absolute_paths)]
    const fn config(&self) -> &crate::config::Config {
        &self.config
    }

    const fn db(&self) -> &Database {
        &self.db
    }
}

#[allow(clippy::absolute_paths)]
// i should replace this with anyhow::Error
type Error = Box<dyn std::error::Error + Send + Sync>;

type DiscordToken = String;

#[tokio::main]
async fn main() {
    // this *should* only need to happen during the config loading,
    // but init_stdout has a sneaky env var call
    dotenv::dotenv().unwrap();

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
                prefix: Some(config.bot.prefix().to_owned()),
                ..Default::default()
            },
            ..Default::default()
        },
        shard_manager: Mutex::new(None),
    };
    poise::set_qualified_names(&mut handler.options.commands);

    #[allow(clippy::shadow_reuse)]
    let handler = Arc::new(handler);
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
            http,
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
        #[allow(clippy::min_ident_chars)]
        let (d, h, m, s) = (
            self.num_days(),
            self.num_hours(),
            self.num_minutes(),
            self.num_seconds(),
        );

        match (d, h, m, s) {
            (1, _, _, _) => ("1 day").to_owned(),
            (2.., _, _, _) => format!("{d} days"),
            (_, 1, _, _) => ("1 hour").to_owned(),
            (_, 2.., _, _) => format!("{h} hours"),
            (_, _, 1, _) => ("1 minute").to_owned(),
            (_, _, 2.., _) => format!("{m} minutes"),
            (_, _, _, 1) => ("1 second").to_owned(),
            (_, _, _, 2..) => format!("{s} seconds"),
            (_, _, _, _) => "less than a second".to_owned(),
        }
    }

    fn format_full(&self) -> String {
        let mut formatted = String::new();

        if self.num_days() > 0 {
            write!(&mut formatted, "{}d ", self.num_days()).unwrap();
        }

        if self.num_hours() > 0 {
            write!(
                &mut formatted,
                "{}h ",
                self.num_hours() - (self.num_days() * 24)
            )
            .unwrap();
        }

        if self.num_minutes() > 0 {
            write!(
                &mut formatted,
                "{}m",
                self.num_minutes() - (self.num_hours() * 60)
            )
            .unwrap();
        } else {
            formatted = "less than a minute".to_owned();
        }

        formatted
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::DateTime;

    #[test]
    fn format_full() {
        #![allow(clippy::unwrap_used)]
        let start = DateTime::parse_from_rfc3339("2024-01-19T20:00:00.000Z").unwrap();

        let end = DateTime::parse_from_rfc3339("2024-01-21T21:19:00.000Z").unwrap();

        let duration = end - start;

        assert_eq!("2d 1h 19m", duration.format_full(),)
    }

    #[test]
    fn format_largest() {
        #![allow(clippy::unwrap_used)]
        let start = DateTime::parse_from_rfc3339("2024-01-19T20:00:00.000Z").unwrap();
        let end = DateTime::parse_from_rfc3339("2024-01-21T21:19:00.000Z").unwrap();
        let duration = end - start;
        assert_eq!("2 days", duration.format_largest(),);

        let start2 = DateTime::parse_from_rfc3339("2024-01-19T20:00:00.000Z").unwrap();
        let end2 = DateTime::parse_from_rfc3339("2024-01-19T21:19:00.000Z").unwrap();
        let duration2 = end2 - start2;
        assert_eq!("1 hour", duration2.format_largest(),);

        let start3 = DateTime::parse_from_rfc3339("2024-01-19T20:00:00.000Z").unwrap();
        let end3 = DateTime::parse_from_rfc3339("2024-01-19T20:19:00.000Z").unwrap();
        let duration3 = end3 - start3;
        assert_eq!("19 minutes", duration3.format_largest(),);
    }
}
