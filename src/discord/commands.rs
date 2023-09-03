mod watch_fic;

use poise::serenity_prelude::Member;
use tracing::{error, info};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, crate::Data, Error>;

#[poise::command(slash_command)]
pub async fn watch_fic(ctx: Context<'_>) -> Result<(), Error> {
    let reply = ctx
        .send(|f| f
            .content("boop")
            .components(|f| f
                .create_action_row(|f| f
                    .create_button(|b| b
                        .label("oop")
                        .custom_id("foo")
                    )
                )
            )
        ).await?;

    let interaction = reply
        .message()
        .await?
        .await_component_interaction(ctx)
        .author_id(ctx.author().id)
        .await;
    
    Ok(())
}

#[tracing::instrument(skip_all)]
#[poise::command(slash_command)]
pub async fn ping(ctx: Context<'_>)  -> Result<(), Error> {
    info!("called by user {}", ctx.author().name);

    ctx.say("pong!").await?;

    Ok(())
}

#[tracing::instrument(skip_all)]
#[poise::command(slash_command)]
pub async fn pfp(
    ctx: Context<'_>,
    user: Option<Member>,
    global: Option<bool>,
) -> Result<(), Error> {
    info!("called by user {}", ctx.author().name);

    if let Err(_) = ctx.defer().await {
        error!("failed to defer - lag will cause errors!")
    }

    let target = match user {
        Some(user) => user,
        None => ctx.author_member().await.unwrap().into_owned(),
    };

    enum PfpType {
        Guild,
        Global,
        Unset,
    }

    // required args are ugly
    let global = global.map_or(false, |b| b);

    let (pfp, pfp_type) = match global {
        true => (
            target.user.face(),
            target
                .user
                .avatar_url()
                .map_or(PfpType::Unset, |_| PfpType::Global),
        ),
        false => (
            target.face(),
            target
                .user
                .avatar_url()
                .map_or(PfpType::Unset, |_| PfpType::Guild),
        ),
    };

    let flavor_text = match pfp_type {
        PfpType::Guild => format!("**{}'s profile picture:**", target.display_name()),
        PfpType::Global => format!("**`{}`'s global profile picture:**", target.user.name),
        PfpType::Unset => format!(
            "**{} does not have a profile picture set!**",
            target.display_name()
        ),
    };

    ctx.send(|f| f.content(flavor_text).attachment((*pfp).into()))
        .await?;

    Ok(())
}