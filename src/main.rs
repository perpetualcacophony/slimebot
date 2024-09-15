#![warn(clippy::perf)]
#![warn(clippy::unwrap_used)]
#![feature(macro_metavar_expr)]
#![feature(let_chains)]
#![feature(associated_type_defaults)]
#![feature(try_blocks)]
#![feature(min_specialization)]
#![feature(const_option)]
#![feature(duration_constructors)]

/// Functionality called from Discord.
mod discord;

mod errors;

mod utils;
use thisslime::TracingError;
use utils::Context;
use utils::Result;

use poise::serenity_prelude::{self as serenity, GatewayIntents};

#[allow(unused_imports)]
use tracing::{debug, info, trace};

mod framework;
use framework::data::PoiseData;

mod built_info {
    // The file has been placed there by the build script.
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

mod commands;

#[cfg(feature = "cli")]
mod cli;

#[cfg(feature = "cli")]
use cli::Cli;

#[tokio::main]
async fn main() {
    let result: Result<()> = try {
        dotenvy::dotenv().ok();

        framework::logging::init_tracing();

        #[cfg(feature = "cli")]
        use clap::Parser;

        #[cfg(feature = "cli")]
        let cli = Cli::parse();

        async fn start(#[cfg(feature = "cli")] cli: &Cli) -> Result<()> {
            let build = if built_info::DEBUG {
                let branch = built_info::GIT_HEAD_REF
                    .map(|s| s.trim_start_matches("/refs/heads/"))
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

            let config = framework::Config::setup(
                #[cfg(feature = "cli")]
                &cli,
            )
            .await?;

            let client = serenity::Client::builder(config.token(), GatewayIntents::all());

            if let Some(flavor_text) = config.logs.flavor_text() {
                info!("{flavor_text}")
            }

            let framework = framework::poise::build(config);

            let mut client = client
                .framework(framework)
                .await
                .expect("client should be valid");

            client
                .start()
                .await
                .expect("client should not return error");

            Ok(())
        }

        #[cfg(feature = "cli")]
        if cli.command.is_start() {
            start(&cli).await?;
        }

        #[cfg(not(feature = "cli"))]
        start().await?;
    };

    if let Err(err) = result {
        err.trace()
    } else {
        tracing::error!("process should not terminate!")
    }
}
