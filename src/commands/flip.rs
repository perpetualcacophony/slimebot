use rand::Rng;
use tracing::instrument;

use crate::utils::{
    poise::{CommandResult, ContextExt},
    Context,
};

#[instrument(skip_all)]
#[poise::command(
    slash_command,
    prefix_command,
    discard_spare_arguments,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
pub async fn flip(
    ctx: Context<'_>,
    coins: Option<usize>,
    #[flag] verbose: bool,
) -> crate::Result<()> {
    let result: CommandResult = try {
        let _typing = ctx.defer_or_broadcast().await?;

        let results = flip_coins(coins.unwrap_or(1));

        ctx.reply_ext(text(&results, verbose)).await?;
    };

    result?;

    Ok(())
}

fn flip_coins(coins: usize) -> Vec<bool> {
    let mut rng = rand::thread_rng();

    let mut new = Vec::with_capacity(coins);

    for _ in 0..coins {
        new.push(rng.gen())
    }

    new
}

fn text(results: &[bool], verbose: bool) -> std::borrow::Cow<str> {
    if results.len() == 1 {
        return if results[0] { "heads" } else { "tails" }.into();
    };

    let heads = results.iter().filter(|heads| **heads).count();
    let tails = results.len() - heads;

    let results_text = format!("{heads} heads & {tails} tails");

    if verbose {
        let vec: Vec<_> = results
            .iter()
            .map(|heads| if *heads { "heads" } else { "tails" })
            .collect();

        format!("**{results_text}** ({verbose})", verbose = vec.join(", ")).into()
    } else {
        results_text.into()
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_str_eq;

    use super::text;

    #[test]
    fn one_coin() {
        assert_str_eq!(text(&[true], false), "heads")
    }

    #[test]
    fn one_coin_verbose() {
        assert_str_eq!(text(&[false], false), "tails")
    }

    #[test]
    fn five_coins() {
        assert_str_eq!(
            text(&[true, false, true, false, false], false),
            "2 heads & 3 tails"
        )
    }

    #[test]
    fn five_coins_verbose() {
        assert_str_eq!(
            text(&[false, false, true, true, false], true),
            "**2 heads & 3 tails** (tails, tails, heads, heads, tails)"
        )
    }
}
