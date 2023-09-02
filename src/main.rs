use anyhow::anyhow;
use poise::serenity_prelude::{self as serenity, json::json, ChannelId, Member, User};
use scraper::{Html, Selector};
use std::{env, fs, str::FromStr, time::Duration};
use tracing::{error, info};

mod log;

struct Data {}
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

// i am not using this yet.
// still figuring out poise
// but poise will break without it
// coconut.jpg
#[poise::command(slash_command)]
async fn intertwined(_ctx: Context<'_>) -> Result<(), Error> {
    /*
    let mut stored_chapter_count = read_chapter_count(intertwined)?;
    let mut chapter_ids = get_chapter_ids(intertwined).await?;

    loop {
        if stored_chapter_count < chapter_ids.len() {
            ctx.say(chapter_ids.last().unwrap().to_string()).await?;
        }

        std::thread::sleep(std::time::Duration::from_millis(10000))
    }*/

    Ok(())
}

#[poise::command(slash_command)]
async fn pfp(
    ctx: Context<'_>,
    user: Option<Member>,
    global: Option<bool>,
) -> Result<(), Error> {
    let target = match user {
        Some(user) => user,
        None => ctx.author_member().await.unwrap().into_owned(),
    };

    enum PfpType {
        Guild,
        Global,
        Unset,
    }

    // required args are ugly
    let global = global.map_or(false, |b| b);

    let (pfp, pfp_type) = match global {
        true => (
            target.user.face(),
            target
                .user
                .avatar_url()
                .map_or(PfpType::Unset, |_| PfpType::Global),
        ),
        false => (
            target.face(),
            target
                .user
                .avatar_url()
                .map_or(PfpType::Unset, |_| PfpType::Guild),
        ),
    };

    let flavor_text = match pfp_type {
        PfpType::Guild => format!("**{}'s profile picture:**", target.display_name()),
        PfpType::Global => format!("**`{}`'s global profile picture:**", target.user.name),
        PfpType::Unset => format!(
            "**{} does not have a profile picture set!**",
            target.display_name()
        ),
    };

    ctx.send(|f| f.content(flavor_text).attachment((*pfp).into()))
        .await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().unwrap();

    // i really should decouple the console logging functionality from discord.
    // like, these panic, because half the code is reliant on a discord connection
    // but they really shouldn't
    let log_channel: u64 = env::var("LOG_CHANNEL").unwrap().parse().unwrap();
    let discord_token = env::var("DISCORD_TOKEN").unwrap();

    // tbh, i don't love initializing the bot this early
    // but it needs the connection for logs to work
    // maybe i can do some stuff with layers? not terribly optimistic
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![pfp()],
            ..Default::default()
        })
        .token(discord_token)
        .intents(serenity::GatewayIntents::non_privileged())
        .setup(|ctx, _, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build()
        .await
        .unwrap();

    // i don't like how far in you have to go to access this :<
    let http = framework.client().cache_and_http.http.clone();

    log::DiscordSubscriber::init(http.clone(), log_channel);

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
