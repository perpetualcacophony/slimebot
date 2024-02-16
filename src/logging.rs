use tracing_subscriber::{
    prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt, EnvFilter,
};

#[allow(unused_imports)]
use tracing::{error, instrument, trace};

#[instrument]
pub fn init_tracing() {
    tracing_subscriber::registry()
    .with(tracing_subscriber::fmt::layer())
    .with(
        EnvFilter::try_new("slimebot,tracing_unwrap")
            .expect("hard-coded env filter should be valid")
    )
    .init();

    trace!("finished");
}