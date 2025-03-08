use poise::PrefixFrameworkOptions;
use tracing::trace;

use crate::{
    commands,
    errors::{self, CommandError, Error},
    utils::serenity::channel::ChannelIdExt,
};

use super::{config::ConfigSetup, data::PoiseData, event_handler};

pub fn build(config: ConfigSetup) -> poise::Framework<PoiseData, Error> {
    poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: commands::list(),
            prefix_options: PrefixFrameworkOptions {
                prefix: Some(config.bot.prefix().to_string()),
                ..Default::default()
            },
            on_error: errors::handle_framework_error,
            event_handler: event_handler::poise,
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                let ctx = ctx.clone();
                let http = ctx.http.clone();

                let commands = framework.options().commands.as_ref();

                if let Some(guild_id) = config.bot.testing_server() {
                    poise::builtins::register_in_guild(&http, commands, *guild_id)
                        .await
                        .expect("registering commands in guild should not fail");
                }

                poise::builtins::register_globally(&http, commands)
                    .await
                    .expect("registering commands globally should not fail");

                let activity = config.bot.activity();
                ctx.set_activity(activity);

                trace!("finished setup, accepting commands");

                if let Some(status_channel) = config.bot.status_channel() {
                    if config.cli.notify_on_start() {
                        status_channel
                            .say_ext(http, "ready!")
                            .await
                            .map_err(CommandError::from)?;
                    }
                }

                Ok(PoiseData::new(config).await?)
            })
        })
        .build()
}
