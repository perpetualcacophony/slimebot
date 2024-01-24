mod ban;
mod watch_fic;

use chrono::Utc;
use poise::serenity_prelude::{Channel, Member, User};
use tracing::{error, info, instrument};

type Context<'a> = poise::Context<'a, crate::Data, BotError>;

pub use watch_fic::watch_fic;

use crate::{BotError, FormatDuration, UtcDateTime};

trait LogCommands {
    async fn log_command(&self);
}

impl LogCommands for Context<'_> {
    async fn log_command(&self) {
        let channel = self
            .channel_id()
            .name(self.cache())
            .await
            .map_or("dms".to_string(), |c| format!("#{c}"));
        info!(
            "@{} ({}): {}",
            self.author().name,
            channel,
            self.invocation_string()
        );
    }
}

/// bot will respond on successful execution
#[instrument(skip_all)]
#[poise::command(slash_command, prefix_command, discard_spare_arguments)]
pub async fn ping(ctx: Context<'_>) -> Result<(), BotError> {
    ctx.log_command().await;

    let ping = ctx.ping().await.as_millis();
    ctx.say(ping_message(ping, "pong")).await?;

    Ok(())
}

fn ping_message(ping: u128, reply: &str) -> String {
    if ping == 0 {
        format!("{reply}! (please try again later to display latency)")
    } else {
        format!("{reply}! ({ping}ms)")
    }
}

#[instrument(skip_all)]
#[poise::command(slash_command, prefix_command, hide_in_help, discard_spare_arguments)]
pub async fn pong(ctx: Context<'_>) -> Result<(), BotError> {
    ctx.log_command().await;

    let ping = ctx.ping().await.as_millis();
    ctx.say(ping_message(ping, "ping")).await?;

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
) -> Result<(), BotError> {
    ctx.log_command().await;

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
) -> Result<(), BotError> {
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
pub async fn ban(ctx: Context<'_>, user: User, reason: Option<String>) -> Result<(), BotError> {
    if ctx.author().id == 497014954935713802 || user.id == 966519580266737715 {
        ban::joke_ban(ctx, ctx.author(), 966519580266737715, "sike".to_string()).await?;
    } else {
        ban::joke_ban(ctx, &user, ctx.author().id.0, reason).await?;
    }

    Ok(())
}

#[instrument(skip(ctx))]
#[poise::command(prefix_command)]
pub async fn banban(ctx: Context<'_>) -> Result<(), BotError> {
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
pub async fn uptime(ctx: Context<'_>) -> Result<(), BotError> {
    ctx.log_command().await;

    ctx.say(format_uptime(ctx.data().started, Utc::now()))
    .await
    .unwrap();

    Ok(())
}

fn format_uptime(started: UtcDateTime, current: UtcDateTime) -> String {
    let uptime = current - started;

    format!(
        "uptime: {} (since {})",
        uptime.format_full(),
        started.format("%Y-%m-%d %H:%M UTC")
    )
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    
    #[test]
    fn ping_message() {
        assert_eq!(
            super::ping_message(0, "pong"),
            "pong! (please try again later to display latency)"
        );

        assert_eq!(
            super::ping_message(128, "pong"),
            "pong! (128ms)"
        );

        assert_eq!(
            super::ping_message(0, "ping"),
            "ping! (please try again later to display latency)"
        );

        assert_eq!(
            super::ping_message(128, "ping"),
            "ping! (128ms)"
        );
    }

    #[test]
    fn format_uptime() {
        assert_eq!(
            super::format_uptime(
                "2024-01-24 07:55:55.457121090Z".parse().unwrap(),
                "2024-01-26 09:05:55.457121090Z".parse().unwrap()
            ),
            "uptime: 2d 1h 10m (since 2024-01-24 07:55 UTC)"
        )
    }
}