mod watch_fic;

use poise::serenity_prelude::{CacheHttp, Channel, Member, User, Webhook, Embed, Color, AttachmentType};
use reqwest::Url;
use serde_json::json;
use tracing::{debug, error, info, instrument, warn};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, crate::Data, Error>;

use tracing_unwrap::ResultExt;
pub use watch_fic::watch_fic;

/// Responds on successful execution.

#[instrument(skip_all)]
#[poise::command(slash_command, prefix_command)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    info!("called by user `{}`", ctx.author().name);

    ctx.say("pong!").await?;

    Ok(())
}

#[instrument(skip_all, fields(author = ctx.author().name, global = global))]
#[poise::command(prefix_command, slash_command)]
pub async fn pfp(
    ctx: Context<'_>,
    user: Option<Member>,
    global: Option<bool>,
) -> Result<(), Error> {
    info!("{}", ctx.invocation_string());
    info!("called by user `{}`", ctx.author().name);

    debug!("{:?}", ctx.guild_id());

    if let Err(_) = ctx.defer().await {
        error!("failed to defer - lag will cause errors!")
    }

    let user = match user {
        Some(user) => user,
        None => ctx.author_member().await.unwrap().into_owned()
    };

    //debug!("target nickname: {}, username: {}", target.display_name(), target.user.name);

    enum PfpType {
        Guild,
        Global,
        Unset,
    }

    //let member = ctx.guild().unwrap().member(ctx.http(), user.user.id);

    // required args are ugly
    let global = global.map_or(false, |b| b);

    let (pfp, pfp_type) = match global {
        true => (
            user.user.face(),
            user
                .avatar_url()
                .map_or(PfpType::Unset, |_| PfpType::Global),
        ),
        false => (
            user.face(),
            user
                .user
                .avatar_url()
                .map_or(PfpType::Unset, |_| PfpType::Guild),
        ),
    };

    let flavor_text = match pfp_type {
        PfpType::Guild => format!("**{}'s profile picture:**", user.display_name()),
        PfpType::Global => format!("**`{}`'s global profile picture:**", user.user.name),
        PfpType::Unset => format!(
            "**{} does not have a profile picture set!**",
            user.display_name()
        ),
    };

    ctx.send(|f| f.content(flavor_text).attachment((*pfp).into()))
        .await?;

    Ok(())
}

#[instrument(skip(ctx))]
#[poise::command(slash_command)]
pub async fn echo(
    ctx: Context<'_>,
    channel: Option<Channel>,
    message: String,
) -> Result<(), Error> {
    let id = match channel {
        Some(channel) => channel.id(),
        None => ctx.channel_id(),
    };

    id.say(ctx.http(), message).await?;

    Ok(())
}

/*#[poise::command(slash_command)]
pub async fn audio(
    ctx: Context<'_>,
) -> Result<(), Error> {
    let manager = songbird::get(ctx.serenity_context()).await.unwrap();

    manager.join(ctx.guild_id().unwrap(), 1098746787868712983).await;
    
    if let Some(handler_lock) = manager.get(ctx.guild_id().unwrap()) {
        let mut handler = handler_lock.lock().await;

        //let mus = tokio::fs::read(
        //    "/home/kate/music/toe/Our Latest Number/02 The Latest Number.flac"
        //).await.unwrap();

        //println!("{mus:?}");

        let mut speaker = espeaking::initialize(None).unwrap().lock();

        let mus = speaker.synthesize("the quick brown fox jumps over the lazy dog");

        let source = Input::new(
            true,
            Reader::from_memory(mus),
            Codec::Pcm,
            Container::Raw,
            None
        );

        //let yt = songbird::ytdl("https://www.youtube.com/watch?v=LvbcIeR36Ro").await.unwrap();

        //println!("{yt:?}");

        handler.play_source(source);
    }

    Ok(())
}*/

#[instrument(skip(ctx, user))]
#[poise::command(prefix_command)]
pub async fn ban(
    ctx: Context<'_>,
    user: User,
    reason: Option<String>
) -> Result<(), Error> {
    if ctx.author().id == 497014954935713802 || user.id == 966519580266737715 {
        joke_ban(ctx, ctx.author(), 966519580266737715, "sike".to_string()).await?;
    } else {
        joke_ban(ctx, &user, ctx.author().id.0, reason).await?;
    }

    Ok(())
}

async fn joke_ban(ctx: Context<'_>, user: &User, moderator_id: u64, reason: impl Into<Option<String>>) -> Result<(), Error> {
    let reason = reason.into().unwrap_or("No reason".to_string());
    
    let embed = ban_embed(&reason, &moderator_id, &user.name);
    let webhook = wick_webhook(ctx).await;
    webhook.execute(ctx.http(), false, |w| w.embeds(vec![embed])).await?;

    Ok(())
}

async fn wick_webhook(ctx: Context<'_>) -> Webhook {
    let wick = ctx.http().get_member(1098746787050836100, 536991182035746816).await.unwrap_or_log();

    let mut hook = ctx.http().get_channel_webhooks(ctx.channel_id().0)
        .await
        .unwrap()
        .into_iter()
        .find(|wh| wh.name == Some("Wick".to_string()))
        .unwrap_or(async { 
            warn!("no webhook for channel {}, creating", ctx.channel_id().as_ref());
            ctx.http().create_webhook(
                *ctx.channel_id().as_u64(),
                &json!({
                    "name": wick.display_name().as_ref()
                }),
                None
            ).await.unwrap_or_log()
        }.await);

    if &hook.name.clone().unwrap() != wick.display_name().as_ref() {
        hook.edit_name(
            ctx.http(),
            wick.display_name().as_ref()
        ).await.unwrap_or_log()
    }

    if hook.avatar.clone().is_none() || hook.avatar.clone().unwrap() != wick.face() {
        hook.edit_avatar(
            ctx.http(),
            AttachmentType::Image(Url::parse(&wick.face()).unwrap())
        ).await.unwrap_or_log()
    }
    
    hook
}

fn ban_embed(reason: &str, moderator_id: &u64, user: &str) -> serde_json::value::Value {
    Embed::fake(|e|
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
    )
}

#[instrument(skip(ctx))]
#[poise::command(prefix_command)]
pub async fn banban(
    ctx: Context<'_>
) -> Result<(), Error> {
    if ctx.author().id == 497014954935713802 {
        joke_ban(ctx, ctx.author(), 966519580266737715, "get banbanned lol".to_string()).await?;
    } else {
        ctx.send(|m| m.attachment(
            "https://cdn.discordapp.com/attachments/1098748818104791122/1167856331940691978/image.png?ex=654fa5f7&is=653d30f7&hm=aec68049fc65377e003368104426b7a19a8e54897fe02de0bf96d95d019b2610&"
            .into()
        )).await.ok();
    }

    Ok(())
}