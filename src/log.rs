use poise::serenity_prelude::{ChannelId, Http};
use std::fmt::Write;
use std::sync::Arc;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tracing::Subscriber;
use tracing_subscriber::{
    prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt, Layer,
};

pub struct DiscordSubscriber;

struct DiscordSender {
    http: Arc<Http>,
    channel: u64,
    rx: UnboundedReceiver<String>,
}

impl DiscordSubscriber {
    pub fn init(http: Arc<Http>, channel: u64) {
        let (tx, rx) = unbounded_channel();
        tracing_subscriber::registry()
            .with(DiscordLayer { tx })
            .with(tracing_subscriber::EnvFilter::from_default_env())
            .init();

        tokio::spawn(DiscordSender::new(http, channel, rx).start());
    }
}

impl DiscordSender {
    async fn start(mut self) {
        println!("spawning sender");

        while let Some(message) = self.rx.recv().await {
            ChannelId(self.channel)
                .say(&self.http, &message)
                .await
                .expect("log failed to reach discord");

            //self.http.send_message(self.channel, &Value::String("m".to_string())).await.ok();

            println!("sent to discord: {message}");
        }
    }

    fn new(http: Arc<Http>, channel: u64, rx: UnboundedReceiver<String>) -> Self {
        Self { http, channel, rx }
    }
}

struct DiscordLayer {
    tx: UnboundedSender<String>,
}

impl<S: Subscriber> Layer<S> for DiscordLayer {
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let mut visitor = DiscordVisitor::default();
        event.record(&mut visitor);

        let message = format!("`[{}] {}`", event.metadata().level(), visitor.message);

        self.tx.send(message).expect("subscriber threading failed")
    }
}

#[derive(Default)]
struct DiscordVisitor {
    message: String,
}

impl tracing::field::Visit for DiscordVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        write!(self.message, "{:?}", value).unwrap();
        println!("record_debug: {}: {:?}", field.name(), value)
        //write!(self.message, "fgjfdkjdgfhj").unwrap()
        //println!("recorder: {:?}", self.message)
    }
}
