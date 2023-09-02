use scraper::{Html, Selector};
use anyhow::{anyhow};
use std::{env, fs, time::Duration};
use poise::serenity_prelude::{self as serenity, json::{Value, json}, ChannelId, Http};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender, UnboundedReceiver};
use tracing::{info, error};

struct Data {}
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[poise::command(slash_command)]
async fn intertwined(ctx: Context<'_>) -> Result<(), Error> {
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

#[tokio::main]
async fn main() {
    dotenv::dotenv().unwrap();

    let log_channel: u64 = env::var("LOG_CHANNEL").unwrap().parse().unwrap();
    let intertwined_role: u64 = env::var("INTERTWINED_ROLE").unwrap().parse().unwrap();
    let intertwined_channel: u64 = env::var("INTERTWINED_CHANNEL").unwrap().parse().unwrap();

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
        .await.unwrap();
        //.build()
        //.await?;

    let http = framework.client().cache_and_http.http.clone();

    DiscordSubscriber::init(http.clone(), log_channel);

    //for i in 0..10 {
    //    ChannelId(1098748588273713184).say(&http, "...").await.unwrap();
    //    tracing::error!("boop {i}");
    //    tokio::time::sleep(Duration::from_millis(2000)).await
    //}

    //tracing::error!("error!");

    let intertwined = 48499684;

    loop {
        let stored_chapter_count = read_chapter_count(intertwined).unwrap();
        let chapter_ids = get_chapter_ids(intertwined).await.unwrap();

        if stored_chapter_count < chapter_ids.len() {
            info!("request made. update!");
            store_chapter_count(intertwined, chapter_ids.len()).await.unwrap();

            ChannelId(intertwined_channel).say(
                &http,
                format!(
                    "<@&{intertwined_role}> **Intertwined has updated!**
                    chapter {}: https://archiveofourown.org/works/{intertwined}/chapters/{}",
                    chapter_ids.len(),
                    chapter_ids.last().unwrap().to_string()
                )
            ).await.unwrap();
        } else {
            info!("request made. no update")
        }

        std::thread::sleep(std::time::Duration::from_secs(60));
    }
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

use std::sync::Arc;

struct DiscordSubscriber {
    tx: UnboundedSender<String>,
}

struct DiscordSender {
    http: Arc<Http>,
    channel: u64,
    rx: UnboundedReceiver<String>
}

impl DiscordSubscriber {
    fn init(http: Arc<Http>, channel: u64) {
        let (tx, rx) = unbounded_channel();
        tracing_subscriber::registry()
            .with(DiscordLayer { tx })
            .with(tracing_subscriber::EnvFilter::from_default_env())
            .init();
            
        tokio::spawn(
            DiscordSender::new(http, channel, rx).start()
        );
    }
    
}

impl DiscordSender {
    async fn start(mut self) {
        println!("spawning sender");

        while let Some(message) = self.rx.recv().await {
            ChannelId(self.channel).say(
                &self.http,
                &message
            ).await.expect("log failed to reach discord");

            //self.http.send_message(self.channel, &Value::String("m".to_string())).await.ok();

            println!("sent to discord: {message}");
        }
    }

    fn new(http: Arc<Http>, channel: u64, rx: UnboundedReceiver<String>) -> Self {
        Self {
            http,
            channel,
            rx
        }
    }
}

use tracing::Subscriber;
use tracing_subscriber::{Layer, prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt};
use std::fmt::Write;

struct DiscordLayer {
    tx: UnboundedSender<String>
}

impl<S: Subscriber> Layer<S> for DiscordLayer {
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        let mut visitor = DiscordVisitor::default();
        event.record(&mut visitor);

        let message = format!(
            "`[{}] {}`",
            event.metadata().level(),
            visitor.message
        );
        
        self.tx.send(message).expect("subscriber threading failed")
    }
}

#[derive(Default)]
struct DiscordVisitor{
    message: String
}

impl tracing::field::Visit for DiscordVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        write!(self.message, "{:?}", value).unwrap();
        println!("record_debug: {}: {:?}", field.name(), value)
        //write!(self.message, "fgjfdkjdgfhj").unwrap()
        //println!("recorder: {:?}", self.message)
    }
}