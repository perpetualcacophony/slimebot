use poise::serenity_prelude::{ChannelId, Http};
use std::fmt::Write;
use std::sync::Arc;
use tokio::sync::{
    mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    oneshot::Sender,
};
use tracing::{error, instrument, trace, Subscriber};
use tracing_subscriber::{
    prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer,
};

pub struct DiscordSubscriber;

impl DiscordSubscriber {
    pub fn init_stdout() -> UnboundedReceiver<String> {
        let (tx, rx) = unbounded_channel();

        tracing_subscriber::registry()
            .with(DiscordLayer::new(tx))
            .with(tracing_subscriber::fmt::layer())
            .with(EnvFilter::try_new("slimebot,tracing_unwrap").unwrap())
            .init();

        let span = tracing::info_span!("init_stdout");
        span.in_scope(|| trace!("stdout logging set up"));

        rx
    }

    #[instrument(skip(http, rx))]
    pub async fn init_discord(http: Arc<Http>, channel: u64, rx: UnboundedReceiver<String>) {
        // just to nail down the multithreading, let's create a disposable channel...
        let (confirm_tx, confirm_rx) = tokio::sync::oneshot::channel();

        // initialize the discord logger with the sender...
        tokio::spawn(DiscordSender::new(http.clone(), channel, rx).start(confirm_tx));

        // and await the response (which doesn't really matter) once it's ready
        confirm_rx.await.ok();

        trace!("discord logging set up");
    }
}

#[derive(Debug)]
pub struct DiscordSender {
    http: Arc<Http>,
    channel: u64,
    rx: UnboundedReceiver<String>,
}

impl DiscordSender {
    #[instrument(skip_all, name = "DiscordSender::start", fields(channel = self.channel))]
    pub async fn start(mut self, oneshot: Sender<bool>) {
        // i don't know any other way to clear the discord buffer lmao
        while self.rx.try_recv().is_ok() {}

        oneshot.send(true).unwrap();

        while let Some(message) = self.rx.recv().await {
            if ChannelId::new(self.channel)
                .say(&self.http, &message)
                .await
                .is_err()
            {
                error!(no_discord = true, "log failed to reach discord");
            }
        }
    }

    pub fn new(http: Arc<Http>, channel: u64, rx: UnboundedReceiver<String>) -> Self {
        Self { http, channel, rx }
    }
}

struct DiscordLayer {
    tx: UnboundedSender<String>,
}

impl DiscordLayer {
    pub const fn new(tx: UnboundedSender<String>) -> Self {
        Self { tx }
    }
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

        if event.metadata().fields().field("no_discord").is_none() {
            self.tx.send(message).expect("subscriber threading failed");
        }
    }
}

#[derive(Default)]
struct DiscordVisitor {
    message: String,
}

impl tracing::field::Visit for DiscordVisitor {
    fn record_debug(&mut self, _field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        write!(self.message, "{value:?}").unwrap();
    }
}
