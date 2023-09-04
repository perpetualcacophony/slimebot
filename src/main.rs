mod logging;
use logging::DiscordSubscriber;

mod discord;
use discord::commands::{ping, pfp, watch_fic};

use anyhow::anyhow;
use poise::serenity_prelude::{self as serenity, ChannelId, GuildId};
use scraper::{Html, Selector};
use std::{env, fs, time::Duration};
use tracing::{info, error, debug, trace};
use tracing_unwrap::{ResultExt, OptionExt};

#[derive(Debug)]
pub struct Data {}

type Error = Box<dyn std::error::Error + Send + Sync>;

#[tokio::main]
async fn main() {
    dotenv::dotenv().unwrap();

    let rx = DiscordSubscriber::init_stdout();

    //tracing_log::LogTracer::init().unwrap_or_log();

    trace!("hi!");

    let bot_token = env::var("BOT_TOKEN").expect_or_log("no BOT_TOKEN in environment");

    // i really should decouple the console logging functionality from discord.
    // like, these panic, because half the code is reliant on a discord connection
    // but they really shouldn't
    let log_channel: u64 = env::var("LOG_CHANNEL").unwrap().parse().unwrap();
    let testing_server: u64 = env::var("TESTING_SERVER").unwrap().parse().unwrap();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![ping(), pfp(), watch_fic()],
            ..Default::default()
        })
        .token(bot_token)
        .intents(serenity::GatewayIntents::non_privileged())
        .setup( move |ctx, _, framework| {
            Box::pin(async move {
                poise::builtins::register_in_guild(ctx, &framework.options().commands, GuildId(testing_server)).await?;
                Ok(Data {})
            })
        })
        .build()
        .await
        .unwrap();

    // i don't like how far in you have to go to access this :<
    let http = framework.client().cache_and_http.http.clone();

    DiscordSubscriber::init_discord(http.clone(), log_channel, rx).await;

    trace!("hi discord!");

    // i think this is an okay pattern?
    // it's probably a bad idea for *all* of the bot's
    // functionality to be defined by command responses.
    // right now it's silly though. the ao3 pinger
    // *should* be a command handler.
    tokio::spawn(framework.start());

    let intertwined_role: u64 = env::var("INTERTWINED_ROLE")
        .unwrap_or_default()
        .parse()
        .unwrap();
    let intertwined_channel: u64 = env::var("INTERTWINED_CHANNEL").unwrap().parse().unwrap();
    let intertwined = 48499684;

    loop {
        let stored_chapter_count = read_chapter_count(intertwined).unwrap();
        let chapter_ids = get_chapter_ids(intertwined).await.unwrap();

        if stored_chapter_count < chapter_ids.len() {
            info!("request made. update!");
            store_chapter_count(intertwined, chapter_ids.len())
                .await
                .unwrap();

            ChannelId(intertwined_channel)
                .say(
                    &http,
                    format!(
                        "<@&{intertwined_role}> **Intertwined has updated!**
                    chapter {}: https://archiveofourown.org/works/{intertwined}/chapters/{}",
                        chapter_ids.len(),
                        chapter_ids.last().unwrap()
                    ),
                )
                .await
                .unwrap();
        } else {
            info!("request made. no update")
        }

        std::thread::sleep(Duration::from_secs(300));
    }
}

fn _has_updated(work_id: usize, current_chapter_count: usize) -> Result<bool, Error> {
    let stored_chapter_count: usize = read_chapter_count(work_id)?;

    //let current_chapter_count = get_chapter_count(work_id)?;

    match stored_chapter_count.cmp(&current_chapter_count) {
        std::cmp::Ordering::Less => Ok(true),
        std::cmp::Ordering::Equal => Ok(false),
        std::cmp::Ordering::Greater => Err(anyhow!("chapter count not stored properly").into()),
    }
}

// i've removed a few more performant ao3 hooks in favor of this one
// i'm a big fan of ao3 and since they don't have an api i want to minimize expensive calls
// it's a little bit of runtime overhead but nbd
async fn get_chapter_ids(work_id: usize) -> Result<Vec<usize>, Error> {
    let work_index = format!("https://archiveofourown.org/works/{}/navigate", work_id);

    let html = reqwest::get(work_index)
        .await
        .unwrap()
        .text()
        .await
        .expect("ao3 request failed");
    let doc = Html::parse_document(&html);
    let selector = Selector::parse("ol.chapter.index.group>li>a").unwrap();

    let chapter_ids = doc
        .select(&selector)
        .map(|el| {
            el.value()
                .attr("href")
                .unwrap()
                .split("/chapters/")
                .nth(1)
                .unwrap()
                .parse()
                .unwrap()
        })
        .collect::<Vec<usize>>();

    Ok(chapter_ids)
}

// this method's annoying to work with
// it's tied to, like, an explicit work right? why supply the chapter count?
// if i could work with an api, i probably *would* have this func call ao3
// but. i can't. i think i've minimized ao3 calls to, like, 1 every loop
async fn store_chapter_count(work_id: usize, chapter_count: usize) -> Result<(), Error> {
    fs::write(format!("works/{work_id}.len"), chapter_count.to_string())?;

    Ok(())
}

fn read_chapter_count(work_id: usize) -> Result<usize, Error> {
    Ok(fs::read_to_string(format!("works/{}.len", work_id))?.parse()?)
}
