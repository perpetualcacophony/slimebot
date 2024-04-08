use super::CommandResult;
use crate::errors::{self, CommandError};
use crate::functions::misc::{self, DiceRoll};
use crate::Context;
use tracing::{debug, instrument};

#[instrument(skip_all)]
#[poise::command(
    slash_command,
    prefix_command,
    rename = "8ball",
    discard_spare_arguments,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
pub async fn eightball(ctx: Context<'_>) -> CommandResult {
    use rand::prelude::thread_rng;

    let answer = misc::ANSWERS.get(&mut thread_rng());
    ctx.reply(answer).await?;

    Ok(())
}

#[instrument(skip_all)]
#[poise::command(
    slash_command,
    prefix_command,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
pub async fn roll(ctx: Context<'_>, #[rest] text: String) -> CommandResult {
    let mut roll: DiceRoll = misc::DiceRoll::parse(&text).map_err(errors::InputError::DiceRoll)?;
    let roll2 = roll.clone();

    let rolls = roll.rolls();
    let total = roll.total();

    let faces = roll.dice.next().expect("at least one die").faces;

    let total = if faces.get() == 1 || (faces.get() == 2 && rolls.clone().count() == 1) {
        total.to_string()
    } else {
        match total {
            t if t == roll2.clone().min() || t == roll2.clone().max() => format!("__{t}__"),
            other => other.to_string(),
        }
    };

    debug!(total);

    let text = if roll.extra == 0 {
        if roll.dice.len().get() == 1 {
            format!("**{total}**")
        } else {
            #[allow(clippy::collapsible_else_if)]
            let roll_text = if faces.get() > 2 {
                rolls
                    .map(|n| match n.get() {
                        n if n == 1 || n == faces.get() => format!("__{n}__"),
                        _ => n.to_string(),
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            } else {
                rolls.map(|n| n.to_string()).collect::<Vec<_>>().join(", ")
            };

            format!("**{total}** ({roll_text})")
        }
    } else {
        let extra = match roll.extra {
            n if n > 0 => format!(", +{n}"),
            n if n < 0 => format!(", {n}"),
            _ => unreachable!(),
        };

        #[allow(clippy::collapsible_else_if)]
        let roll_text = if faces.get() > 2 {
            rolls
                .map(|n| match n.get() {
                    n if n == 1 || n == faces.get() => format!("__{n}__"),
                    _ => n.to_string(),
                })
                .collect::<Vec<_>>()
                .join(", ")
        } else {
            rolls.map(|n| n.to_string()).collect::<Vec<_>>().join(", ")
        };

        format!("**{total}** ({roll_text}{extra})")
    };

    ctx.reply(text).await?;

    Ok(())
}

#[instrument(skip_all)]
#[poise::command(
    slash_command,
    prefix_command,
    discard_spare_arguments,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
pub async fn d20(ctx: Context<'_>) -> CommandResult {
    let _typing = ctx.defer_or_broadcast().await?;

    let die = misc::Die::d20();
    let rolled = die.roll().get();

    ctx.reply(format!("**{rolled}**")).await?;

    Ok(())
}
