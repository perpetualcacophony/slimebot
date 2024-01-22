mod ban;
mod watch_fic;

use poise::serenity_prelude::{Channel, Member, User};
use tokio::join;
use tracing::{error, info, instrument};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, crate::Data, Error>;

pub use watch_fic::watch_fic;

use crate::FormatDuration;

/// bot will respond on successful execution
#[instrument(skip_all)]
#[poise::command(slash_command, prefix_command, discard_spare_arguments)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    let (channel, ping) = join!(ctx.channel_id().name(ctx.cache()), ctx.ping(),);

    info!(
        "@{} ({}): {}",
        ctx.author().name,
        channel.map_or("dms".to_string(), |c| format!("#{c}")),
        ctx.invocation_string()
    );

    let ping = ping.as_millis();
    if ping == 0 {
        ctx.say("pong! (please try again later to display latency)")
            .await?;
    } else {
        ctx.say(format!("pong! ({}ms)", ping)).await?;
    }

    Ok(())
}

#[instrument(skip_all)]
#[poise::command(slash_command, prefix_command, hide_in_help, discard_spare_arguments)]
pub async fn pong(ctx: Context<'_>) -> Result<(), Error> {
    let (channel, ping) = join!(ctx.channel_id().name(ctx.cache()), ctx.ping(),);

    info!(
        "@{} ({}): {}",
        ctx.author().name,
        channel.map_or("dms".to_string(), |c| format!("#{c}")),
        ctx.invocation_string()
    );

    let ping = ping.as_millis();
    if ping == 0 {
        ctx.say("ping! (please try again later to display latency)")
            .await?;
    } else {
        ctx.say(format!("ping! ({}ms)", ping)).await?;
    }

    Ok(())
}

/// display a user's profile picture
#[instrument(skip_all)]
#[poise::command(prefix_command, slash_command, discard_spare_arguments, rename = "pfp")]
pub async fn pfp(
    ctx: Context<'_>,
    #[description = "the user to display the profile picture of - defaults to you"] user: Option<
        User,
    >,
    #[flag]
    #[description = "show the user's global profile picture, ignoring if they have a server one set"]
    global: bool,
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

    if ctx.defer().await.is_err() {
        error!("failed to defer - lag will cause errors!");
    }

    if let Some(guild) = ctx.guild() {
        let member = if let Some(user) = user {
            guild.member(ctx.http(), user.id).await.unwrap()
        } else {
            ctx.author_member().await.unwrap().into_owned()
        };

        enum PfpType {
            Guild,
            GlobalOnly,
            Global,
            Unset,
        }
        use PfpType as P;

        let (pfp, pfp_type) = if global {
            (
                member.user.face(),
                member.user.avatar_url().map_or_else(
                    || PfpType::Unset,
                    |_| {
                        member
                            .avatar_url()
                            .map_or(PfpType::GlobalOnly, |_| PfpType::Global)
                    },
                ),
            )
        } else {
            (
                member.face(),
                member.avatar_url().map_or_else(
                    || {
                        member
                            .user
                            .avatar_url()
                            .map_or(PfpType::Unset, |_| PfpType::Global)
                    },
                    |_| PfpType::Guild,
                ),
            )
        };

        fn author_response(pfp_type: PfpType, global: bool) -> String {
            match pfp_type {
                P::Guild => "**your profile picture in this server:**",
                P::GlobalOnly => "**your profile picture:**",
                P::Global if global => "**your global profile picture:**",
                P::Global => "**your profile picture:**",
                P::Unset if global => "**you don't have a profile picture set!**",
                P::Unset => "**you don't have a profile picture set!**",
            }
            .to_string()
        }

        fn other_response(member: &Member, pfp_type: PfpType, global: bool) -> String {
            match pfp_type {
                P::Guild => format!(
                    "**{}'s profile picture in this server:**",
                    member.display_name()
                ),
                P::GlobalOnly => format!("**`{}`'s profile picture:**", member.user.name),
                P::Global if global => {
                    format!("**`{}`'s global profile picture:**", member.user.name)
                }
                P::Global => format!("**{}'s profile picture:**", member.display_name()),
                P::Unset if global => format!(
                    "**`{}` does not have a profile picture set!**",
                    member.user.name
                ),
                P::Unset => format!(
                    "**{} does not have a profile picture set!**",
                    member.display_name()
                ),
            }
        }

        let response_text = if &member.user == ctx.author() {
            author_response(pfp_type, global)
        } else {
            other_response(&member, pfp_type, global)
        };

        ctx.send(|f| f.content(response_text).attachment((*pfp).into()))
            .await?;
    } else {
        fn author_response(author: &User) -> (String, String) {
            let response_text = if author.avatar_url().is_some() {
                "**your profile picture:**"
            } else {
                "**you don't have a profile picture set!**"
            }
            .to_string();

            (author.face(), response_text)
        }

        fn other_response(user: &User) -> (String, String) {
            let response_text = if user.avatar_url().is_some() {
                format!("**`{}`'s profile picture:**", user.name)
            } else {
                format!("**`{}` does not have a profile picture set!**", user.name)
            };

            (user.face(), response_text)
        }

        let (pfp, response_text) = if let Some(user) = user {
            if &user == ctx.author() {
                author_response(ctx.author())
            } else {
                other_response(&user)
            }
        } else {
            author_response(ctx.author())
        };

        ctx.send(|f| f.content(response_text).attachment((*pfp).into()))
            .await?;
    }

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
        ctx.send(|m| m.content("https://files.catbox.moe/jm6sr9.png"))
            .await
            .unwrap();
    }

    Ok(())
}

#[instrument(skip(ctx))]
#[poise::command(prefix_command)]
pub async fn uptime(ctx: Context<'_>) -> Result<(), Error> {
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

    let started = ctx.data().started;
    let uptime = chrono::Utc::now() - started;

    ctx.say(format!(
        "uptime: {} (since {})",
        uptime.format_full(),
        started.format("%Y-%m-%d %H:%M UTC")
    ))
    .await
    .unwrap();

    Ok(())
}
