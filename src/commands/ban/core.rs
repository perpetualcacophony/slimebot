use poise::serenity_prelude::{
    Color, CreateAttachment, CreateEmbed, EditWebhook, ExecuteWebhook, User, Webhook,
};
use serde_json::json;
use tracing::warn;
use tracing_unwrap::ResultExt;

use crate::errors::CommandError as Error;
use crate::utils::Context;

pub async fn joke_ban(
    ctx: Context<'_>,
    user: &User,
    moderator_id: u64,
    reason: impl Into<Option<String>>,
) -> Result<(), Error> {
    let reason = reason.into().unwrap_or_else(|| "No reason".to_string());

    let embed = ban_embed(&reason, moderator_id, &user.name);
    let webhook = wick_webhook(ctx).await;
    webhook
        .execute(ctx.http(), false, ExecuteWebhook::new().embeds(vec![embed]))
        .await?;

    Ok(())
}

async fn wick_webhook(ctx: Context<'_>) -> Webhook {
    let wick = ctx
        .http()
        .get_member(1098746787050836100.into(), 536991182035746816.into())
        .await
        .unwrap_or_log();

    let mut hook = ctx
        .http()
        .get_channel_webhooks(ctx.channel_id())
        .await
        .expect("webhooks request should not fail")
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
                        ctx.channel_id(),
                        &json!({
                            "name": wick.display_name()
                        }),
                        None,
                    )
                    .await
                    .unwrap_or_log()
            }
            .await,
        );

    if hook.name.clone().expect("webhook name should be valid") != wick.display_name() {
        hook.edit(ctx.http(), EditWebhook::new().name(wick.display_name()))
            .await
            .unwrap_or_log();
    }

    if hook.avatar.clone().is_none() || hook.avatar != wick.avatar {
        hook.edit(
            ctx.http(),
            EditWebhook::new().avatar(
                &CreateAttachment::url(ctx.http(), &wick.face())
                    .await
                    .expect("creating attachment should not fail"),
            ),
        )
        .await
        .unwrap_or_log();
    }

    hook
}

fn ban_embed(reason: &str, moderator_id: u64, user: &str) -> CreateEmbed {
    CreateEmbed::default()
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
}
