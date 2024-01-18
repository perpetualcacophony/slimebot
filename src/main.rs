/// Logging frontends, with [`tracing`](https://docs.rs/tracing/latest/tracing/) backend.
mod logging;
use chrono::{DateTime, Utc};
use logging::DiscordSubscriber;

/// Functionality called from Discord.
mod discord;
use discord::commands::*;
use discord::framework::Handler;
use mongodb::Database;
use serde::{Deserialize, Serialize};

/// Config file parsing and option access.
mod config;

mod db;

use poise::{
    serenity_prelude::{self as serenity, GatewayIntents},
    PrefixFrameworkOptions,
};
use std::time::{Duration, SystemTime};
use tracing::{error, info, trace};
use tracing_unwrap::ResultExt;

#[derive(Debug)]
pub struct Data {
    config: config::Config,
    db: Database,
}

impl Data {
    async fn new() -> Self {
        let config: crate::config::Config = ::config::Config::builder()
            .add_source(::config::File::with_name("slimebot.toml"))
            .add_source(::config::Environment::with_prefix("SLIMEBOT"))
            .build()
            .expect_or_log("config file could not be loaded")
            .try_deserialize()
            .expect_or_log("configuration could not be parsed");

        let db = db::connect(&config.db).await;

        Self { config, db }
    }

    fn config(&self) -> &crate::config::Config {
        &self.config
    }

    fn db(&self) -> &Database {
        &self.db
    }
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct MoreData {
    last_vore_mention: DateTime<Utc>,
}

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
    // now the first log can be sent!
    trace!("hi!");

    let data = Data::new().await;
    let config = data.config.clone();

    let mut handler = Handler {
        data,
        options: poise::FrameworkOptions {
            commands: vec![ping(), pfp(), watch_fic(), echo(), ban(), banban()],
            prefix_options: PrefixFrameworkOptions {
                prefix: Some(config.bot.prefix().to_string()),
                ..Default::default()
            },
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

    // i think this is an okay pattern?
    // it's probably a bad idea for *all* of the bot's
    // functionality to be defined by command responses.
    // right now it's silly though. the ao3 pinger
    // *should* be a command handler.
    tokio::spawn(async move { client.start().await });

    trace!("discord framework started");

    let mut keep_alive = tokio::time::interval(Duration::from_secs(600));
    keep_alive.tick().await;
    loop {
        let before = SystemTime::now();
        if http.get_bot_gateway().await.is_err() {
            error!("failed to connect to discord!")
        }
        let ping = SystemTime::now()
            .duration_since(before)
            .unwrap()
            .as_millis();
        info!("discord connection active! ({ping}ms)");

        keep_alive.tick().await;
    }
}
