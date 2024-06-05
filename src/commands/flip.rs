use rand::{Rng, SeedableRng};
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
pub async fn flip(ctx: Context<'_>, coins: Option<u8>, #[flag] verbose: bool) -> crate::Result<()> {
    _flip(ctx, coins, verbose).await?;
    Ok(())
}

async fn _flip(ctx: Context<'_>, coins: Option<u8>, verbose: bool) -> CommandResult {
    let _typing = ctx.defer_or_broadcast().await?;

    let coins = coins.map(|int| if int == 0 { 1 } else { int }).unwrap_or(1);

    let mut rng = rand::rngs::StdRng::from_rng(rand::thread_rng()).expect("valid rng");

    // extremely simple processing for 1 flip
    let text = if coins == 1 {
        let heads: bool = rng.gen();

        if heads {
            "heads".to_owned()
        } else {
            "tails".to_owned()
        }
    } else {
        let mut heads = 0;
        let mut tails = 0;
        // small optimization - allocate `coins` capacity if verbose, or 0 if not
        let mut results = Vec::with_capacity(verbose.then_some(coins).unwrap_or_default().into());

        for _ in 0..coins {
            if rng.gen() {
                heads += 1;

                if verbose {
                    results.push("heads")
                }
            } else {
                tails += 1;

                if verbose {
                    results.push("tails")
                }
            }
        }

        let results_text = format!("{heads} heads & {tails} tails");

        let verbose_text = if verbose {
            format!("({})", results.join(", "))
        } else {
            "".to_owned()
        };

        if verbose {
            format!("**{results_text}** {verbose_text}")
        } else {
            results_text
        }
    };

    ctx.reply_ext(text).await?;

    Ok(())
}
