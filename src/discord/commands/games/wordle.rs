use std::ops::Not;

use poise::serenity_prelude::{CreateMessage, User};
use poise::CreateReply;
use tracing::{debug, instrument};

use super::super::utils::CommandResult;
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
    subcommands("daily", "random", "display")
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
    wordles.refresh(wordle.words()).await?;
    let mut playable = wordles.playable_for(ctx.author().id).await?;

    if let Some(daily) = playable.next() {
        if let Some(message_id) = ctx.data().wordle().game_in_channel(ctx.channel_id()).await {
            ctx.reply_ephemeral(format!(
                "there's already a game being played in this channel! {}",
                message_id.link(ctx.channel_id(), ctx.guild_id()),
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

            wordle.add_game(message.channel_id, message.id).await;

            // play game
            let mut game = wordle::Game::new(
                ctx,
                &mut message,
                ctx.data().wordle.words(),
                wordles,
                daily.puzzle,
                style,
            );

            game.setup().await?;
            game.run().await?;

            wordle.remove_game(ctx.channel_id()).await;
        }
    } else {
        ctx.reply_ephemeral("you don't have a daily wordle available!")
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

    debug!(?wordle.active_games);

    if let Some(msg) = wordle.game_in_channel(ctx.channel_id()).await {
        ctx.reply_ephemeral(format!(
            "there's already a game being played in this channel! {}",
            msg.link(ctx.channel_id(), ctx.guild_id()),
        ))
        .await?;

        return Ok(());
    } else {
        let mut game_msg = ctx.reply("loading...").await?.into_message().await?;
        wordle.add_game(ctx.channel_id(), game_msg.id).await;

        let puzzle = wordle::Puzzle::random(wordle.words());

        let mut game = wordle::Game::new(
            ctx,
            &mut game_msg,
            wordle.words(),
            wordle.wordles(),
            puzzle,
            style,
        );

        game.setup().await?;

        // play game
        game.run().await?;

        wordle.remove_game(ctx.channel_id()).await;
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
