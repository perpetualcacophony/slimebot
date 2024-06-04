#![warn(clippy::perf)]
#![warn(clippy::unwrap_used)]
#![feature(macro_metavar_expr)]
#![feature(let_chains)]
#![feature(associated_type_defaults)]

/// Functionality called from Discord.
mod discord;
#[allow(clippy::wildcard_imports)]
use mongodb::Database;

mod errors;

mod functions;

mod utils;
use utils::Context;

use poise::{
    serenity_prelude::{self as serenity, GatewayIntents},
    PrefixFrameworkOptions,
};

#[allow(unused_imports)]
use tracing::{debug, info, trace};

mod framework;
use framework::data::PoiseData;

use functions::games::wordle::{game::GamesCache, DailyWordles, WordsList};

use crate::utils::serenity::channel::ChannelIdExt;

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

mod built_info {
    // The file has been placed there by the build script.
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

#[tokio::main]
async fn main() {
    framework::logging::init_tracing();

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

    let data = PoiseData::new();
    let config = data.config.clone();

    if let Some(flavor_text) = config.logs.flavor_text() {
        info!("{flavor_text}")
    }

    let framework = framework::poise::build(data);

    let mut client = serenity::Client::builder(config.bot.token(), GatewayIntents::all())
        .framework(framework)
        .await
        .expect("client should be valid");

    client
        .start()
        .await
        .expect("client should not return error");
}
