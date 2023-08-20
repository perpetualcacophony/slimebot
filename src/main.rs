use scraper::{Html, Selector};
use anyhow::{anyhow};
use tokio::sync::mpsc;
use tracing::span::AsId;
use std::{env, fs};
use poise::serenity_prelude::{self as serenity, json::{Value, json}, ChannelId, Http};
use log::{info, error};
use tokio::sync::mpsc::{Sender, Receiver};

const INTERTWINED_ROLE: u64 = 1142348135079891005;

struct Data {}
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[poise::command(slash_command)]
async fn intertwined(ctx: Context<'_>) -> Result<(), Error> {
    let intertwined = 48499684;
    let mut stored_chapter_count = read_chapter_count(intertwined)?;
    let mut chapter_ids = get_chapter_ids(intertwined).await?;

    loop {
        if stored_chapter_count < chapter_ids.len() {
            ctx.say(chapter_ids.last().unwrap().to_string()).await?;
        }
        
        std::thread::sleep(std::time::Duration::from_millis(10000))
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv::dotenv()?;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![intertwined()],
            ..Default::default()
        })
        .token(std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN"))
        .intents(serenity::GatewayIntents::non_privileged())
        .setup(|_, _, _| Box::pin(async move {
            Ok(Data {  })
        }))
        .build()
        .await?;
        //.build()
        //.await?;

    let (tx, mut rx) = mpsc::channel(1);

    let http = framework.client().cache_and_http.http.clone();
    
    let logger = DiscordLogger::new(http.clone(), tx, 1142654870487302184);
    let log2 = logger.clone();

    tokio::spawn(log2.sender(rx));
    
    log::set_boxed_logger(Box::new(logger))
        .map(|()| log::set_max_level(LevelFilter::Trace))?;

    error!("boop");
    println!("booped");

        /*.setup(|ctx, _ready, framework| async {
            ctx.http.send_message(1141851433029861427, &Value::String("boop".to_string()))
                .await;
        
            Box::pin(async move {
                poise::builtins::register_in_guild(ctx, &framework.options().commands, 1098746787050836100.into()).await?;
                Ok(Data {})
            })
        });*/

    //ChannelId(1098748588273713184).say(http, "whatever").await?;

    let intertwined = 48499684;
    let mut stored_chapter_count = read_chapter_count(intertwined)?;
    let mut chapter_ids = get_chapter_ids(intertwined).await?;
    let mut looper = 0;

    loop {
        if looper >= 12 {
            stored_chapter_count = read_chapter_count(intertwined)?;
            chapter_ids = get_chapter_ids(intertwined).await?;
            looper = 0;

            if stored_chapter_count < chapter_ids.len() {
                ChannelId(1141851433029861427).say(
                    &http,
                    format!(
                        "<@&{INTERTWINED_ROLE}> **Intertwined has updated!**
                        chapter {}: https://archiveofourown.org/works/{intertwined}/chapters/{}",
                        chapter_ids.len(),
                        chapter_ids.last().unwrap().to_string()
                    )
                ).await?;
    
                store_chapter_count(intertwined, chapter_ids.len()).await?;
                stored_chapter_count = read_chapter_count(intertwined)?;
                println!("request made. update!")
            } else {
                println!("request made. no update")
            }
        } else {
            looper += 1
        }

        http.broadcast_typing(1141851433029861427).await?;

        std::thread::sleep(std::time::Duration::from_millis(5000));
    }

    //http.send_message(1098748588273713184, &json!({"message": "bar"})).await?;

    //framework.start().await?;

    println!("homos");

    //let msg = http.http.send_message(1098748588273713184, &Value::String("boop".to_string())).await;

    //println!("{:?}", msg);

    //let work_id = env::args().nth(1).unwrap().parse()?;
    //let current_chapter_count = get_chapter_count(work_id)?;

    //dbg!(has_updated(work_id, current_chapter_count)?);

    Ok(())
}

fn has_updated(work_id: usize, current_chapter_count: usize) -> Result<bool, Error> {
    let stored_chapter_count: usize = read_chapter_count(work_id)?;
        
    //let current_chapter_count = get_chapter_count(work_id)?;

    match stored_chapter_count.cmp(&current_chapter_count) {
        std::cmp::Ordering::Less => Ok(true),
        std::cmp::Ordering::Equal => Ok(false),
        std::cmp::Ordering::Greater => Err(anyhow!("chapter count not stored properly").into()),
    }
}

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

    let chapter_ids =
        doc
            .select(&selector)
            .map(|el|
                el
                    .value()
                    .attr("href")
                    .unwrap()
                    .split("/chapters/")
                    .nth(1)
                    .unwrap()
                    .parse()
                    .unwrap()
                )
            .collect::<Vec<usize>>();
    
    Ok(chapter_ids)
}

fn get_chapter_count(work_id: usize) -> Result<usize, Error> {
    let work_index = format!("https://archiveofourown.org/works/{}/navigate", work_id);
    
    let html = reqwest::blocking::get(work_index)
        .unwrap()
        .text()?;
    let doc = Html::parse_document(&html);
    let selector = Selector::parse("ol.chapter.index.group>li>a").unwrap();

    let chapter_count =
        doc
            .select(&selector)
            .count();
    
    Ok(chapter_count)
}

async fn store_chapter_count(work_id: usize, chapter_count: usize) -> Result<(), Error> {
    fs::write(
        format!("works/{work_id}.len"),
        get_chapter_ids(work_id).await?.len().to_string()
    )?;

    Ok(())
}

fn read_chapter_count(work_id: usize) -> Result<usize, Error> {
    Ok(fs::read_to_string(format!("works/{}.len", work_id))?.parse()?)
}

fn get_latest_chapter_id(work_id: usize) -> Result<usize, Error> {
    let work_index = format!("https://archiveofourown.org/works/{}/navigate", work_id);
    
    let html = reqwest::blocking::get(work_index)
        .unwrap()
        .text()?;
    let doc = Html::parse_document(&html);
    let selector = Selector::parse("ol.chapter.index.group>li>a").unwrap();

    let chapter_id =
        doc
            .select(&selector)
            .last()
            .unwrap()
            .value()
            .attr("href")
            .unwrap()
            .split("/chapters/")
            .nth(1)
            .unwrap()
            .parse()
            .unwrap();
    
    Ok(chapter_id)
}

//use tokio::sync::Mutex;
use std::sync::Arc;

#[derive(Clone)]
struct DiscordLogger {
    http: Arc<Http>,
    tx: Sender<String>,
    channel: u64,
}

use log::{SetLoggerError, LevelFilter, Log};

impl DiscordLogger {
    async fn sender(self, mut rx: Receiver<String>) {
        while let Some(message) = rx.recv().await {
            ChannelId(self.channel).say(
                &self.http,
                message
            ).await.expect("logging failure - this is very bad!");
        }
    }

    fn new(http: Arc<Http>, tx: Sender<String>, channel: u64) -> Self {
        Self {
            http,
            tx,
            channel
        }
    }

    fn init(http: Arc<Http>, tx: Sender<String>, channel: u64) -> Result<(), SetLoggerError> {
        let logger = DiscordLogger::new(http, tx, channel);
        log::set_boxed_logger(Box::new(logger))
            .map(|()| log::set_max_level(LevelFilter::Trace)) 
    }

    
}

impl Log for DiscordLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        true
    }

    fn flush(&self) {}

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        
        let message = format!("[{}] {}", record.level(), "boop");

        let tx = self.tx.clone();
        tokio::spawn(async move {tx.send(message).await});
    }
}

use tracing::Subscriber;
use tracing_subscriber::Layer;
use std::fmt::Write;

/*impl tracing::Subscriber for DiscordLogger {
    fn enabled(&self, metadata: &tracing::Metadata<'_>) -> bool {
        true
    }

    fn new_span(&self, span: &span::Attributes<'_>) -> span::Id {
        span
    }
}*/

struct LayerCake;

impl<S: Subscriber> Layer<S> for LayerCake {
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        let mut visitor = DiscordVisitor(String::new());
        event.record(&mut visitor);

        
    }
}

struct DiscordVisitor(String);

impl tracing::field::Visit for DiscordVisitor {
    fn record_debug(&mut self, _field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        write!(self.0, "{:?}", value).unwrap()
    }
}