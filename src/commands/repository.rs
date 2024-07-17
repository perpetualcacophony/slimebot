use crate::utils::poise::{CommandResult, ContextExt};
use crate::utils::Context;

#[poise::command(slash_command, prefix_command, aliases("repo"))]
pub async fn repository(ctx: Context<'_>) -> crate::Result<()> {
    let result: CommandResult = try {
        let repo = ctx.data().config.bot.github_repo()?;
        ctx.reply_ext(repo.to_github_url()).await?;
    };

    result?;

    Ok(())
}
