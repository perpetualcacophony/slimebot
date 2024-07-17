use crate::utils::Context;

#[poise::command(slash_command, prefix_command, aliases("repo"))]
pub async fn repository(ctx: Context<'_>) -> crate::Result<()> {
    Ok(())
}
