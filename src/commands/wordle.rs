use poise::serenity_prelude::{Mentionable, User};
use std::ops::Not;
use tracing::{debug, instrument};

use crate::utils::poise::{CommandResult, Context, ContextExt};
use crate::{errors::SendMessageError, Result};

pub mod core;
use core::{
    self as wordle, core::AsEmoji, game::options::GameOptionsBuilder, game::options::GameStyle,
};

/// play wordle right from discord!
#[instrument(skip_all)]
#[poise::command(
    slash_command,
    prefix_command,
    discard_spare_arguments,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL",
    subcommands("daily", "random", "display", "role", "unused")
)]
pub async fn wordle(ctx: Context<'_>) -> Result<()> {
    _wordle(ctx).await?;
    Ok(())
}

async fn _wordle(ctx: Context<'_>) -> CommandResult {
    //let words = ctx.data().wordle.words();
    //let dailies = ctx.data().wordle.wordles();

    //crate::games::wordle::play(ctx, mode, words.clone(), dailies.clone(), style, fix_flags).await?;

    poise::builtins::help(
        ctx,
        Some("wordle"),
        poise::builtins::HelpConfiguration::default(),
    )
    .await
    .map_err(SendMessageError::from)?;

    Ok(())
}

/// play a daily wordle in DMs
#[instrument(skip_all)]
#[poise::command(
    slash_command,
    prefix_command,
    discard_spare_arguments,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
async fn daily(ctx: Context<'_>, style: Option<GameStyle>) -> Result<()> {
    _daily(ctx, style).await?;
    Ok(())
}

async fn _daily(ctx: Context<'_>, style: Option<GameStyle>) -> CommandResult {
    let wordle = ctx.data().wordle();
    let wordles = wordle.wordles();

    if let Some(new_daily) = wordles.refresh(wordle.words()).await?
        && let Some(channel) = ctx.data().config().wordle.channel_id
        && let Some(role) = ctx.data().config().wordle.role_id
    {
        channel
            .say(
                ctx,
                format!(
                    "{ping} **Daily wordle {number} now available!**\nPlay it with `/wordle daily`",
                    ping = role.mention(),
                    number = new_daily.puzzle.number
                ),
            )
            .await?;
    }

    let mut playable = wordles.playable_for(ctx.author().id).await?;

    if let Some(daily) = playable.next() {
        if let Some(data) = wordle.game_data().get(ctx.channel_id()).await {
            ctx.reply_ephemeral(format!(
                "there's already a game being played in this channel! {}",
                data.message_id.link(ctx.channel_id(), ctx.guild_id()),
            ))
            .await?;

            return Ok(());
        } else {
            // play game
            let mut game = wordle::Game::new(
                ctx,
                daily.puzzle.clone(),
                GameOptionsBuilder::default().style(style).build(),
            )
            .await?;

            game.setup().await?;
            game.run().await?;

            if let Some(completed) = wordle
                .wordles()
                .find_game(ctx.author().id, daily.puzzle.number)
                .await?
                && let Some(channel) = &ctx.data().config().wordle.channel_id
            {
                channel
                    .say(
                        ctx,
                        format!(
                            "`{username}` **completed wordle {number}!**\n{emojis}",
                            username = ctx.author().name,
                            number = daily.puzzle.number,
                            emojis = completed.as_emoji()
                        ),
                    )
                    .await?;
            }
        }
    } else {
        let latest = wordles
            .latest()
            .await?
            .expect("wordle has been refreshed by now");

        ctx.reply_ephemeral(format!(
            "you don't have a daily wordle yet! check back in {hours} hours",
            hours = latest.age_hours()
        ))
        .await?;
    }

    Ok(())
}

/// practice with a random wordle
#[instrument(skip_all)]
#[poise::command(
    slash_command,
    prefix_command,
    discard_spare_arguments,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
async fn random(ctx: Context<'_>, style: Option<GameStyle>) -> Result<()> {
    let result: CommandResult = try {
        let wordle = ctx.data().wordle();

        debug!(data = ?wordle.game_data());

        if let Some(data) = wordle.game_data().get(ctx.channel_id()).await {
            ctx.reply_ephemeral(format!(
                "there's already a game being played in this channel! {}",
                data.message_id.link(ctx.channel_id(), ctx.guild_id()),
            ))
            .await?;

            return Ok(());
        } else {
            let puzzle = wordle::Puzzle::random(wordle.words());

            let mut game = wordle::Game::new(
                ctx,
                puzzle,
                GameOptionsBuilder::default().style(style).build(),
            )
            .await?;

            game.setup().await?;

            // play game
            game.run().await?;
        }
    };

    result?;

    Ok(())
}

/// display your own results for a given wordle, or someone else's
#[instrument(skip_all)]
#[poise::command(
    slash_command,
    prefix_command,
    discard_spare_arguments,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
async fn display(
    ctx: Context<'_>,
    #[description = "the wordle's number"] number: u32,
    #[description = "the user to show results for (defaults to you)"] user: Option<User>,
) -> Result<()> {
    let result: CommandResult = try {
        let _typing = ctx.defer_or_broadcast().await?;

        let wordles = ctx.data().wordle.wordles();

        if wordles.wordle_exists(number).await?.not() {
            ctx.reply_ephemeral("that wordle doesn't exist!").await?;
            return Ok(());
        }

        let user = user.as_ref().unwrap_or_else(|| ctx.author());

        if let Some(game) = wordles.find_game(user.id, number).await? {
            if game.num_guesses == 0 {
                ctx.reply_ephemeral(
                    "that user has started the wordle but hasn't guessed anything!",
                )
                .await?;
            }

            let text = format!(
                "wordle {} (`{}`):\n>>> {}",
                number,
                user.name,
                game.as_emoji()
            );

            ctx.reply_ext(text).await?;
        } else {
            ctx.reply_ephemeral("that user hasn't started that wordle!")
                .await?;
        }
    };

    result?;

    Ok(())
}

/// display your own results for a given wordle, or someone else's
#[instrument(skip_all)]
#[poise::command(
    slash_command,
    prefix_command,
    discard_spare_arguments,
    guild_only,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL | MANAGE_ROLES"
)]
async fn role(ctx: Context<'_>) -> Result<()> {
    let result: CommandResult = try {
        let config = ctx.data().config();

        if let Some(role_id) = &config.wordle.role_id {
            let member = ctx.author_member().await.expect("command is guild-only");
            if member.roles.contains(role_id) {
                member.remove_role(ctx, role_id).await?;
                ctx.reply_ephemeral("Removed the wordle role!").await?;
            } else {
                member.add_role(ctx, role_id).await?;
                ctx.reply_ephemeral("Gave you the wordle role!").await?;
            }
        } else {
            ctx.reply_ephemeral("Error: there's no wordle role configured!")
                .await?;
        }
    };

    result?;

    Ok(())
}

/* #[instrument(skip_all)]
#[poise::command(
    slash_command,
    prefix_command,
    discard_spare_arguments,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
async fn letters(ctx: Context<'_>) -> Result<()> {
    let result: CommandResult = try {
        let wordle = ctx.data().wordle();

        if let Some(data) = wordle.game_data().get(ctx.channel_id()).await {
            let guesses = &data.guesses;

            let response = format!(
                "guessed letters:\n{guessed}\n\nunused letters:\n{unused}",
                guessed = guesses.letter_states().as_emoji(),
                unused = guesses.unused_letters().as_emoji()
            );

            ctx.reply(response).await?;
        } else {
            ctx.reply_ephemeral("there isn't a game active in this channel!")
                .await?;
        }
    };

    result?;

    Ok(())
} */

#[instrument(skip_all)]
#[poise::command(
    slash_command,
    prefix_command,
    discard_spare_arguments,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
async fn unused(ctx: Context<'_>) -> Result<()> {
    let result: CommandResult = try {
        let wordle = ctx.data().wordle();

        if let Some(data) = wordle.game_data().get(ctx.channel_id()).await {
            let guesses = &data.guesses;

            let response = format!(
                "unused letters:\n{unused}",
                unused = guesses.unused_letters().as_emoji()
            );

            ctx.reply(response).await?;
        } else {
            ctx.reply_ephemeral("there isn't a game active in this channel!")
                .await?;
        }
    };

    result?;

    Ok(())
}
