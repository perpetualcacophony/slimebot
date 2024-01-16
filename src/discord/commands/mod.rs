mod ban;
mod watch_fic;

use poise::serenity_prelude::{Channel, Member, User};
use tracing::{error, info, instrument};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, crate::Data, Error>;

pub use watch_fic::watch_fic;

/// Responds on successful execution.

#[instrument(skip_all)]
#[poise::command(slash_command, prefix_command)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    let channel = ctx
        .channel_id()
        .name(ctx.cache())
        .await
        .map_or("dms".to_string(), |c| format!("#{c}"));
    info!(
        "@{} ({}): {}",
        ctx.author().name,
        channel,
        ctx.invocation_string()
    );

    ctx.say("pong!").await?;

    Ok(())
}

#[instrument(skip_all)]
#[poise::command(prefix_command, slash_command)]
pub async fn pfp(
    ctx: Context<'_>,
    user: Option<Member>,
    global: Option<bool>,
) -> Result<(), Error> {
    let channel = ctx
        .channel_id()
        .name(ctx.cache())
        .await
        .map_or("dms".to_string(), |c| format!("#{c}"));
    info!(
        "@{} ({}): {}",
        ctx.author().name,
        channel,
        ctx.invocation_string()
    );

    // debug!("{:?}", ctx.guild_id());

    if ctx.defer().await.is_err() {
        error!("failed to defer - lag will cause errors!")
    }

    let user = match user {
        Some(user) => user,
        None => ctx.author_member().await.unwrap().into_owned(),
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
            user.avatar_url()
                .map_or(PfpType::Unset, |_| PfpType::Global),
        ),
        false => (
            user.face(),
            user.user
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
pub async fn ban(ctx: Context<'_>, user: User, reason: Option<String>) -> Result<(), Error> {
    if ctx.author().id == 497014954935713802 || user.id == 966519580266737715 {
        ban::joke_ban(ctx, ctx.author(), 966519580266737715, "sike".to_string()).await?;
    } else {
        ban::joke_ban(ctx, &user, ctx.author().id.0, reason).await?;
    }

    Ok(())
}

#[instrument(skip(ctx))]
#[poise::command(prefix_command)]
pub async fn banban(ctx: Context<'_>) -> Result<(), Error> {
    if ctx.author().id == 497014954935713802 {
        ban::joke_ban(
            ctx,
            ctx.author(),
            966519580266737715,
            "get banbanned lol".to_string(),
        )
        .await?;
    } else {
        ctx.send(|m| m.attachment("https://files.catbox.moe/jm6sr9.png".into()))
            .await
            .ok();
    }

    Ok(())
}
