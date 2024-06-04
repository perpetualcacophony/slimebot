use poise::PrefixFrameworkOptions;
use tracing::trace;

use crate::{
    discord,
    errors::{self, CommandError},
    utils::serenity::channel::ChannelIdExt,
};

use super::{data::PoiseData, event_handler};

pub fn build(data: PoiseData) -> poise::Framework<PoiseData, CommandError> {
    poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: discord::commands::list(),
            prefix_options: PrefixFrameworkOptions {
                prefix: Some(data.config.bot.prefix().to_string()),
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

                if let Some(guild_id) = data.config.bot.testing_server() {
                    poise::builtins::register_in_guild(&http, commands, *guild_id)
                        .await
                        .expect("registering commands in guild should not fail");
                }

                poise::builtins::register_globally(&http, commands)
                    .await
                    .expect("registering commands globally should not fail");

                let activity = data.config.bot.activity();
                ctx.set_activity(activity);

                trace!("finished setup, accepting commands");

                if let Some(status_channel) = data.config.bot.status_channel() {
                    status_channel.say_ext(http, "ready!").await?;
                }

                Ok(data)
            })
        })
        .build()
}
