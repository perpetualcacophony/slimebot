use poise::serenity_prelude::{AttachmentType, Color, Embed, Error as SerenityError, User, Webhook};
use reqwest::Url;
use serde_json::json;
use tracing::warn;
use tracing_unwrap::ResultExt;

use super::Context;

pub async fn joke_ban(
    ctx: Context<'_>,
    user: &User,
    moderator_id: u64,
    reason: impl Into<Option<String>>,
) -> Result<(), SerenityError> {
    let reason = reason.into().unwrap_or_else(|| "No reason".to_string());

    let embed = ban_embed(&reason, moderator_id, &user.name);
    let webhook = wick_webhook(ctx).await;
    webhook
        .execute(ctx.http(), false, |w| w.embeds(vec![embed]))
        .await?;

    Ok(())
}

async fn wick_webhook(ctx: Context<'_>) -> Webhook {
    let wick = ctx
        .http()
        .get_member(1098746787050836100, 536991182035746816)
        .await
        .unwrap_or_log();

    let mut hook = ctx
        .http()
        .get_channel_webhooks(ctx.channel_id().0)
        .await
        .unwrap()
        .into_iter()
        .find(|wh| wh.name == Some("Wick".to_string()))
        .unwrap_or(
            async {
                warn!(
                    "no webhook for channel {}, creating",
                    ctx.channel_id().as_ref()
                );
                ctx.http()
                    .create_webhook(
                        *ctx.channel_id().as_u64(),
                        &json!({
                            "name": wick.display_name().as_ref()
                        }),
                        None,
                    )
                    .await
                    .unwrap_or_log()
            }
            .await,
        );

    if &hook.name.clone().unwrap() != wick.display_name().as_ref() {
        hook.edit_name(ctx.http(), wick.display_name().as_ref())
            .await
            .unwrap_or_log();
    }

    if hook.avatar.clone().is_none() || hook.avatar.clone().unwrap() != wick.face() {
        hook.edit_avatar(
            ctx.http(),
            AttachmentType::Image(Url::parse(&wick.face()).unwrap()),
        )
        .await
        .unwrap_or_log();
    }

    hook
}

fn ban_embed(reason: &str, moderator_id: u64, user: &str) -> serde_json::value::Value {
    Embed::fake(|e| {
        e
        .title("Ban result:")
        .fields([
            (
                "",
                format!(
                    "<:reason:1167852271560839248> **Reason:** {reason}
                    <:moderator:1167852275868389537> **Moderator:** <@{moderator_id}><:ticket:1167852279383216198><:message:1167852277273464956><:star_ticket:1167852280264003634><:star:1167852409989627954><:bomb:1167852281551663166>
                    <:crosshair:1167852283422314588> **Details:**
                    \u{200B}\u{200B}\u{200B}\u{200B}<:double_arrow:1167852272659734550> Duration: <:fail:1167852407028461648>
                    \u{200B}\u{200B}\u{200B}\u{200B}<:double_arrow:1167852272659734550> Soft Ban: <:fail:1167852407028461648>
                    \u{200B}\u{200B}\u{200B}\u{200B}<:double_arrow:1167852272659734550> Hack Ban: <:fail:1167852407028461648>
                    \u{200B}\u{200B}\u{200B}\u{200B}<:double_arrow:1167852272659734550> DM-Members: <:fail:1167852407028461648>",    
                ),
                false
            ),
            (
                "",
                format!("<:success:1167852408626499664> **Successful bans**
                <:arrow:1167852274589122631> `{user}`"),
                false
            ),
            (
                "",
                "<:fail:1167852407028461648> **Unsuccessful bans**
                All users were banned!".to_string(),
                false
            )
        ])
        .color(Color::from_rgb(47, 49, 54))
    })
}
