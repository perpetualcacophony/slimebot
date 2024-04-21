#![warn(clippy::perf)]
#![warn(clippy::unwrap_used)]
#![feature(macro_metavar_expr)]
#![feature(let_chains)]
#![feature(associated_type_defaults)]

/// Logging frontends, with [`tracing`](https://docs.rs/tracing/latest/tracing/) backend.
mod logging;
use std::{collections::HashMap, sync::Arc};

/// Functionality called from Discord.
mod discord;
use arc_swap::ArcSwap;
#[allow(clippy::wildcard_imports)]
use discord::commands::*;
use mongodb::Database;

/// Config file parsing and option access.
mod config;

mod db;

mod errors;

mod functions;

mod utils;
use utils::Context;

use poise::{
    serenity_prelude::{
        self as serenity, collect, futures::StreamExt, ChannelId, Event, GatewayIntents, MessageId,
    },
    PrefixFrameworkOptions,
};

use tokio::sync::RwLock;
#[allow(unused_imports)]
use tracing::{debug, info, trace};

use tracing_unwrap::ResultExt;

use chrono::Utc;
type UtcDateTime = chrono::DateTime<Utc>;

#[derive(Debug, Clone)]
pub struct Data {
    config: config::Config,
    db: Database,
    started: UtcDateTime,
    wordle: WordleData,
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

        let wordle = WordleData::new(&db);

        Self {
            config,
            db,
            started,
            wordle,
        }
    }

    const fn config(&self) -> &crate::config::Config {
        &self.config
    }

    const fn db(&self) -> &Database {
        &self.db
    }

    const fn wordle(&self) -> &WordleData {
        &self.wordle
    }
}

use functions::games::wordle::{game::GamesCache, DailyWordles, GameData, GameRecord, WordsList};

#[derive(Debug, Clone)]
struct WordleData {
    words: WordsList,
    wordles: DailyWordles,
    game_data: GamesCache,
}

impl WordleData {
    fn new(db: &Database) -> Self {
        let words = WordsList::load();
        let wordles = DailyWordles::new(db);
        let game_data = GamesCache::new();

        Self {
            words,
            wordles,
            game_data,
        }
    }

    const fn words(&self) -> &WordsList {
        &self.words
    }

    const fn wordles(&self) -> &DailyWordles {
        &self.wordles
    }

    const fn game_data(&self) -> &GamesCache {
        &self.game_data
    }
}

type DiscordToken = String;

#[tokio::main]
async fn main() {
    logging::init_tracing();

    let build = if built_info::DEBUG {
        let branch = built_info::GIT_HEAD_REF
            .map(|s| s.split('/').last().expect("head ref should have slashes"))
            .unwrap_or("DETACHED");

        format!(
            "development branch {} (`{}`)",
            branch,
            built_info::GIT_COMMIT_HASH_SHORT.expect("should be built with a git repo")
        )
    } else {
        format!("release {}", built_info::PKG_VERSION)
    };

    info!("{build}");

    let data = Data::new();
    let config = data.config.clone();

    if let Some(flavor_text) = config.logs.flavor_text() {
        info!("{flavor_text}")
    }

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: discord::commands::list(),
            prefix_options: PrefixFrameworkOptions {
                prefix: Some(config.bot.prefix().to_string()),
                ..Default::default()
            },
            on_error: errors::handle_framework_error,
            ..Default::default()
        })
        .setup(|ctx, ready, framework| {
            Box::pin(async move {
                let arc = Arc::new(data.clone());

                let ctx = ctx.clone();
                let shard = ctx.shard.clone();
                let http = ctx.http.clone();

                let bot_id = ready.user.id;

                let commands = &framework.options().commands;
                poise::builtins::register_in_guild(
                    &http,
                    commands.as_ref(),
                    *data
                        .config
                        .bot
                        .testing_server()
                        .expect("bot testing server id should be valid"),
                )
                .await
                .expect("registering commands in guild should not fail");

                let activity = data.config.bot.activity();
                ctx.set_activity(activity);

                let watchers_config = data.config.watchers.clone();
                let messages = collect(&shard, |event| match event {
                    Event::MessageCreate(event) => Some(event.message.clone()),
                    _ => None,
                })
                .filter(move |msg| {
                    let msg = msg.clone();
                    let cache = ctx.cache.clone();
                    let config = &watchers_config;
                    let allowed = config.channel_allowed(msg.channel_id);

                    async move { !msg.is_own(cache) && !msg.is_private() && allowed }
                });

                let messages_http = http.clone();
                let messages_arc = arc.clone();
                let messages_task = messages.for_each(move |msg| {
                    //let http = _ctx.clone().http();
                    let data = messages_arc.clone();
                    let http = messages_http.clone();

                    trace!(?msg.id, "message captured");

                    async move {
                        use discord::watchers::*;

                        tokio::join!(
                            vore(&http, &data.db, &msg),
                            l_biden(&http, &msg),
                            look_cl(&http, &msg),
                            watch_haiku(&http, &msg),
                        );
                    }
                });
                tokio::spawn(messages_task);

                let reactions = collect(&shard, |event| match event {
                    Event::ReactionAdd(event) => Some(event.reaction.clone()),
                    _ => None,
                })
                .filter(move |reaction| {
                    let reaction = reaction.clone();

                    async move { reaction.user_id != Some(bot_id) && reaction.guild_id.is_some() }
                });

                let config = data.config().clone();
                let channel = config.bug_reports_channel().copied();

                let react_http = http.clone();
                if let Some(channel) = channel {
                    let reactions_task = reactions.for_each(move |reaction| {
                        let http = react_http.clone();

                        trace!(?reaction.message_id, "reaction captured");

                        async move {
                            use discord::bug_reports::bug_reports;

                            bug_reports(&http, reaction, &channel).await;
                        }
                    });

                    tokio::spawn(reactions_task);
                }

                trace!("finished setup, accepting commands");

                if let Some(status_channel) = config.bot.status_channel() {
                    status_channel
                        .say(http, "ready!")
                        .await
                        .expect_or_log("failed to send status message");
                }

                Ok(data)
            })
        })
        .build();

    let mut client = serenity::Client::builder(config.bot.token(), GatewayIntents::all())
        .framework(framework)
        .await
        .expect("client should be valid");

    client
        .start()
        .await
        .expect("client should not return error");
}

trait FormatDuration {
    fn format_largest(&self) -> String;
    fn format_full(&self) -> String;
}

impl FormatDuration for chrono::Duration {
    #[rustfmt::skip]
    fn format_largest(&self) -> String {
        let (d, h, m, s) = (
            self.num_days(),
            self.num_hours(),
            self.num_minutes(),
            self.num_seconds(),
        );

        match (d, h, m, s) {
            (1  , _  , _  , _  ) => ("1 day").to_string(),
            (2.., _  , _  , _  ) => format!("{d} days"),
            (_  , 1  , _  , _  ) => ("1 hour").to_string(),
            (_  , 2.., _  , _  ) => format!("{h} hours"),
            (_  , _  , 1  , _  ) => ("1 minute").to_string(),
            (_  , _  , 2.., _  ) => format!("{m} minutes"),
            (_  , _  , _  , 1  ) => ("1 second").to_string(),
            (_  , _  , _  , 2..) => format!("{s} seconds"),
            (_  , _  , _  , _  ) => "less than a second".to_string(),
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

mod built_info {
    // The file has been placed there by the build script.
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::DateTime;
    use pretty_assertions::assert_eq;

    #[test]
    fn format_full() {
        let start = DateTime::parse_from_rfc3339("2024-01-19T20:00:00.000Z")
            .expect("hard-coded timestamp should be valid");

        let end = DateTime::parse_from_rfc3339("2024-01-21T21:19:00.000Z")
            .expect("hard-coded timestamp should be valid");

        let duration = end - start;

        assert_eq!("2d 1h 19m", duration.format_full(),)
    }

    #[test]
    fn format_largest() {
        let start = DateTime::parse_from_rfc3339("2024-01-19T20:00:00.000Z")
            .expect("hard-coded timestamp should be valid");
        let end = DateTime::parse_from_rfc3339("2024-01-21T21:19:00.000Z")
            .expect("hard-coded timestamp should be valid");
        let duration = end - start;
        assert_eq!("2 days", duration.format_largest(),);

        let start = DateTime::parse_from_rfc3339("2024-01-19T20:00:00.000Z")
            .expect("hard-coded timestamp should be valid");
        let end = DateTime::parse_from_rfc3339("2024-01-19T21:19:00.000Z")
            .expect("hard-coded timestamp should be valid");
        let duration = end - start;
        assert_eq!("1 hour", duration.format_largest(),);

        let start = DateTime::parse_from_rfc3339("2024-01-19T20:00:00.000Z")
            .expect("hard-coded timestamp should be valid");
        let end = DateTime::parse_from_rfc3339("2024-01-19T20:19:00.000Z")
            .expect("hard-coded timestamp should be valid");
        let duration = end - start;
        assert_eq!("19 minutes", duration.format_largest(),);
    }
}
