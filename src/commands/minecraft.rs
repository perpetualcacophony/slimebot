use crate::{commands::LogCommands, utils::poise::ContextExt};

use crate::utils::{poise::CommandResult, Context};
use poise::serenity_prelude::{self as serenity};
use tracing::{debug, instrument};

#[instrument(skip_all)]
#[poise::command(
    slash_command,
    prefix_command,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL",
    subcommands("claim")
)]
pub async fn minecraft(ctx: Context<'_>, server: Option<String>) -> crate::Result<()> {
    ctx.log_command().await;

    let result: CommandResult = try {
        let data = ctx.data().minecraft();

        let address = server.unwrap_or("162.218.211.126".to_owned());
        let response = api::Response::get(&address).await.map_err(Error::Api)?;

        match response {
            api::Response::Online(response) => {
                let mut players_fields = Vec::with_capacity(response.players.online);
                for player in &response.players {
                    let description = if let Some(serenity_id) =
                        data.players().player_from_minecraft(player.name()).await?
                    {
                        let user = serenity_id.to_user(ctx).await?;

                        if let Some(guild_id) = ctx.guild_id() {
                            user.nick_in(ctx, guild_id).await.unwrap_or(user.name)
                        } else {
                            user.name
                        }
                    } else {
                        "".to_owned()
                    };

                    players_fields.push((player.name(), description, false))
                }

                let mut embed = serenity::CreateEmbed::new()
                    .title(format!(
                        "{host} ({version})",
                        host = response.host(),
                        version = response.version.name
                    ))
                    .description(format!(
                        "{motd}\n\nplayers online: {count}",
                        count = response.players.online,
                        motd = response.motd.clean()
                    ))
                    .fields(players_fields);

                if let Some(url) = response.icon_url().await.map_err(Error::Api)? {
                    debug!(%url);
                    embed = embed.thumbnail(url.to_string());
                }

                ctx.send_ext(poise::CreateReply::default().reply(true).embed(embed))
                    .await?;
            }
            api::Response::Offline(response) => {
                ctx.say_ext(format!(
                    "{host} is offline!",
                    host = response.host().map(ToString::to_string).unwrap_or(address)
                ))
                .await?;
            }
        }
    };

    result?;

    Ok(())
}

#[instrument(skip_all)]
#[poise::command(
    slash_command,
    prefix_command,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
pub async fn claim(ctx: Context<'_>, username: String) -> crate::Result<()> {
    let result: CommandResult = try {
        let data = ctx.data().minecraft();
        data.players()
            .add_username(ctx.author().id, username.clone())
            .await?
            .map_err(Error::AlreadyClaimed)?;

        ctx.reply_ext(format!(
            "claimed minecraft account {name}!",
            name = username
        ))
        .await?;
    };

    result?;

    Ok(())
}

pub mod api;

mod players;

mod data;
pub use data::Data;

mod error;
pub use error::Error;
