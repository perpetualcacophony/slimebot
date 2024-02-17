#![warn(clippy::perf)]
#![warn(clippy::unwrap_used)]

/// Logging frontends, with [`tracing`](https://docs.rs/tracing/latest/tracing/) backend.
mod logging;
use std::sync::Arc;

/// Functionality called from Discord.
mod discord;
#[allow(clippy::wildcard_imports)]
use discord::commands::*;
use mongodb::Database;

/// Config file parsing and option access.
mod config;

mod db;

use poise::{
    serenity_prelude::{self as serenity, collect, futures::StreamExt, Event, GatewayIntents},
    PrefixFrameworkOptions,
};

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

    const fn config(&self) -> &crate::config::Config {
        &self.config
    }

    const fn db(&self) -> &Database {
        &self.db
    }
}

// i should replace this with anyhow::Error
type Error = Box<dyn std::error::Error + Send + Sync>;

type DiscordToken = String;

#[tokio::main]
async fn main() {
    logging::init_tracing();

    let data = Data::new();
    let config = data.config.clone();

    if let Some(flavor_text) = config.logs.flavor_text() {
        info!("{flavor_text}")
    }

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                ping(),
                pong(),
                pfp(),
                watch_fic(),
                echo(),
                ban(),
                banban(),
                uptime(),
                //purge_after(),
                borzoi(),
                minecraft(),
            ],
            prefix_options: PrefixFrameworkOptions {
                prefix: Some(config.bot.prefix().to_string()),
                ..Default::default()
            },
            ..Default::default()
        })
        .setup(|ctx, ready, framework| {
            Box::pin( async move {
                let data = Data::new();
                let arc = Arc::new(data.clone());

                let ctx = ctx.clone();
                let shard = ctx.shard.clone();
                let http = ctx.http.clone();

                let bot_id = ready.user.id;

                let commands = &framework.options().commands;
                poise::builtins::register_in_guild(
                    &http,
                    commands.as_ref(),
                    *data.config.bot.testing_server().expect("bot testing server id should be valid")
                ).await.expect("registering commands in guild should not fail");

                let activity = data.config.bot.activity();
                ctx.set_activity(activity);

                let messages = collect(&shard, |event| {
                    match event {
                        Event::MessageCreate(event) => Some(event.message.clone()),
                        _ => None,
                    }
                }).filter(
                    move |msg| {
                            let msg = msg.clone();
                            let cache = ctx.cache.clone();

                            async move {
                                !msg.is_own(cache)
                                && !msg.is_private()
                            }
                    }
                );
                
                let messages_http = http.clone();
                let messages_arc = arc.clone();
                let messages_task = messages.for_each(move |msg| {
                    //let http = _ctx.clone().http();
                    let data = messages_arc.clone();
                    let http = messages_http.clone();

                    async move {
                        use discord::watchers::*;

                        tokio::join!(
                            vore(&http, &data.db, &msg),
                            l_biden(&http, &msg),
                            look_cl(&http, &msg),
                        );
                    }
                });
                tokio::spawn(messages_task);

                let reactions = collect(&shard, |event| {
                    match event {
                        Event::ReactionAdd(event) => Some(event.reaction.clone()),
                        _ => None,
                    }
                }).filter(
                    move |reaction| {
                        let reaction = reaction.clone();
                        let data = arc.clone();
                        let config = &data.config.watchers;
                        let channel_allowed = config.channel_allowed(reaction.channel_id);

                        async move {
                            reaction.user_id != Some(bot_id)
                            && reaction.guild_id.is_some()
                            && channel_allowed
                        }
                    }
                );

                let config = data.config().clone();
                let channel = config.bug_reports_channel().copied();

                if let Some(channel) = channel {
                    let reactions_task = reactions.for_each(move |reaction| {
                        let http = http.clone();
    
                        async move {
                            use discord::bug_reports::bug_reports;
    
                            bug_reports(&http, reaction, &channel).await;
                        }
                    });

                    tokio::spawn(reactions_task);
                }

                Ok(data)
            })
        })
        .build();

    let mut client = serenity::Client::builder(config.bot.token(), GatewayIntents::all())
        .framework(framework)
        .await
        .expect("client should be valid");

    trace!("discord framework set up");

    /*let shards = client.shard_manager.clone();

    tokio::spawn(async move {
        loop { 
            let runners = shards.runners.clone();
            let guard = runners.lock().await;    

            let shard = guard.get(&ShardId(0));
            //debug!(?shard);

            if let Some(shard) = shard {
                if shard.stage == ConnectionStage::Connected {

                    let messages = MessageCollector::new(shard);

                    messages.stream().for_each(|msg| {

                        let http = http.clone();
                        let db = data.db.clone();

                        async move {
                            discord::watchers::vore(&http, &db, &msg).await;
                        }   
                    }).await
                }
            }
        }
    });*/

    trace!("discord framework started");
    client.start().await.expect("client should not return error");
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
        let start = DateTime::parse_from_rfc3339("2024-01-19T20:00:00.000Z").expect("hard-coded timestamp should be valid");

        let end = DateTime::parse_from_rfc3339("2024-01-21T21:19:00.000Z").expect("hard-coded timestamp should be valid");

        let duration = end - start;

        assert_eq!("2d 1h 19m", duration.format_full(),)
    }

    #[test]
    fn format_largest() {
        let start = DateTime::parse_from_rfc3339("2024-01-19T20:00:00.000Z").expect("hard-coded timestamp should be valid");
        let end = DateTime::parse_from_rfc3339("2024-01-21T21:19:00.000Z").expect("hard-coded timestamp should be valid");
        let duration = end - start;
        assert_eq!("2 days", duration.format_largest(),);

        let start = DateTime::parse_from_rfc3339("2024-01-19T20:00:00.000Z").expect("hard-coded timestamp should be valid");
        let end = DateTime::parse_from_rfc3339("2024-01-19T21:19:00.000Z").expect("hard-coded timestamp should be valid");
        let duration = end - start;
        assert_eq!("1 hour", duration.format_largest(),);

        let start = DateTime::parse_from_rfc3339("2024-01-19T20:00:00.000Z").expect("hard-coded timestamp should be valid");
        let end = DateTime::parse_from_rfc3339("2024-01-19T20:19:00.000Z").expect("hard-coded timestamp should be valid");
        let duration = end - start;
        assert_eq!("19 minutes", duration.format_largest(),);
    }
}
