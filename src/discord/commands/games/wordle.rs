use std::fmt::Write;
use std::ops::Not;

use poise::serenity_prelude::json::to_string;
use poise::serenity_prelude::{CreateMessage, Mentionable, User};
use poise::CreateReply;
use tracing::{debug, instrument};

use super::super::utils::CommandResult;
use crate::functions::games::wordle::core::guess::GuessSlice;
use crate::functions::games::wordle::core::AsEmoji;
use crate::{discord::utils::ContextExt, utils::Context};

use crate::functions::games::wordle::{self, GameStyle};

/// play wordle right from discord!
#[instrument(skip_all)]
#[poise::command(
    slash_command,
    prefix_command,
    discard_spare_arguments,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL",
    subcommands("daily", "random", "display", "role", "letters", "unused")
)]
pub async fn wordle(ctx: Context<'_>) -> CommandResult {
    //let words = ctx.data().wordle.words();
    //let dailies = ctx.data().wordle.wordles();

    //crate::games::wordle::play(ctx, mode, words.clone(), dailies.clone(), style, fix_flags).await?;

    poise::builtins::help(
        ctx,
        Some("wordle"),
        poise::builtins::HelpConfiguration::default(),
    )
    .await?;

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
async fn daily(ctx: Context<'_>, style: Option<GameStyle>) -> CommandResult {
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
        if let Some(data) = wordle.game_data.get(ctx.channel_id()).await {
            ctx.reply_ephemeral(format!(
                "there's already a game being played in this channel! {}",
                data.message_id.link(ctx.channel_id(), ctx.guild_id()),
            ))
            .await?;

            return Ok(());
        } else {
            let mut message = if ctx.guild_id().is_some() {
                ctx.reply("you can't play a daily wordle in a server - check your dms!")
                    .await?;
                ctx.author()
                    .dm(ctx, CreateMessage::new().content("loading..."))
                    .await?
            } else {
                ctx.reply("loading...").await?.into_message().await?
            };
            // play game
            let mut game = wordle::Game::new(
                ctx,
                &mut message,
                wordle.words(),
                wordles,
                wordle.game_data(),
                daily.puzzle.clone(),
                style,
            );

            game.setup().await?;
            game.run().await?;

            if let Some(completed) = wordle
                .wordles
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
async fn random(ctx: Context<'_>, style: Option<GameStyle>) -> CommandResult {
    let wordle = ctx.data().wordle();

    debug!(?wordle.game_data);

    if let Some(data) = wordle.game_data.get(ctx.channel_id()).await {
        ctx.reply_ephemeral(format!(
            "there's already a game being played in this channel! {}",
            data.message_id.link(ctx.channel_id(), ctx.guild_id()),
        ))
        .await?;

        return Ok(());
    } else {
        let mut game_msg = ctx.reply("loading...").await?.into_message().await?;

        let puzzle = wordle::Puzzle::random(wordle.words());

        let mut game = wordle::Game::new(
            ctx,
            &mut game_msg,
            wordle.words(),
            wordle.wordles(),
            wordle.game_data(),
            puzzle,
            style,
        );

        game.setup().await?;

        // play game
        game.run().await?;
    }

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
) -> CommandResult {
    let _typing = ctx.defer_or_broadcast().await?;

    let wordles = ctx.data().wordle.wordles();

    if wordles.wordle_exists(number).await?.not() {
        ctx.send(
            CreateReply::default()
                .content("that wordle doesn't exist!")
                .reply(true)
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    }

    let user = user.as_ref().unwrap_or_else(|| ctx.author());

    if let Some(game) = wordles.find_game(user.id, number).await? {
        if game.num_guesses == 0 {
            ctx.send(
                CreateReply::default()
                    .content("that user has started the wordle but hasn't guessed anything!")
                    .reply(true)
                    .ephemeral(true),
            )
            .await?;
        }

        let text = format!(
            "wordle {} (`{}`):\n>>> {}",
            number,
            user.name,
            game.as_emoji()
        );

        ctx.reply(text).await?;
    } else {
        ctx.send(
            CreateReply::default()
                .content("that user hasn't started that wordle!")
                .reply(true)
                .ephemeral(true),
        )
        .await?;
    }

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
async fn role(ctx: Context<'_>) -> CommandResult {
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

    Ok(())
}

#[instrument(skip_all)]
#[poise::command(
    slash_command,
    prefix_command,
    discard_spare_arguments,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
async fn letters(ctx: Context<'_>) -> CommandResult {
    let wordle = ctx.data().wordle();

    if let Some(data) = wordle.game_data.get(ctx.channel_id()).await {
        let guesses = &data.guesses;

        let response = format!(
            "guessed letters:\n{guessed}\n\nunused letters:\n{unused}",
            guessed = guesses.letter_states().as_emoji(),
            unused = guesses.unused_letters().as_emoji()
        );

        ctx.reply(response).await?;
    } else {
        ctx.reply("there isn't a game active in this channel!")
            .await?;
    }

    Ok(())
}

#[instrument(skip_all)]
#[poise::command(
    slash_command,
    prefix_command,
    discard_spare_arguments,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
async fn unused(ctx: Context<'_>) -> CommandResult {
    let wordle = ctx.data().wordle();

    if let Some(data) = wordle.game_data.get(ctx.channel_id()).await {
        let guesses = &data.guesses;

        let response = format!(
            "unused letters:\n{unused}",
            unused = guesses.unused_letters().as_emoji()
        );

        ctx.reply(response).await?;
    } else {
        ctx.reply("there isn't a game active in this channel!")
            .await?;
    }

    Ok(())
}
