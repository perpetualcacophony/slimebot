mod ban;
mod watch_fic;

use anyhow::anyhow;
use mongodb::bson::doc;
use poise::{
    serenity_prelude::{
        futures::StreamExt, CacheHttp, Channel, ChannelId, CreateAttachment, CreateButton,
        CreateInteractionResponse, CreateInteractionResponseFollowup,
        CreateInteractionResponseMessage, CreateMessage, EditMessage, Member, Message, MessageId,
        ReactionType, User,
    },
    CreateReply,
};
use rand::{Rng, SeedableRng};
use serde::Deserialize;

use tracing::trace;
#[allow(unused_imports)]
use tracing::{debug, error, info, instrument};

//type Error = Box<dyn std::error::Error + Send + Sync>;
type Error = errors::Error;
type Context<'a> = poise::Context<'a, crate::Data, Error>;

type CommandResult = Result<(), Error>;

pub use watch_fic::watch_fic;

use crate::{
    built_info,
    discord::commands::roll::DiceRoll,
    errors::{self, InputError},
    roll::Die,
    wordle::{AsEmoji, Game},
    FormatDuration,
};

trait LogCommands {
    async fn log_command(&self);
}

impl LogCommands for Context<'_> {
    async fn log_command(&self) {
        let channel = self
            .channel_id()
            .name(self.http())
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
#[poise::command(
    slash_command,
    prefix_command,
    discard_spare_arguments,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
pub async fn ping(ctx: Context<'_>) -> CommandResult {
    ctx.log_command().await;

    let ping = ctx.ping().await.as_millis();
    if ping == 0 {
        ctx.say("pong! (please try again later to display latency)")
            .await?;
    } else {
        ctx.say(format!("pong! ({}ms)", ping)).await?;
    }

    Ok(())
}

#[instrument(skip_all)]
#[poise::command(
    slash_command,
    prefix_command,
    hide_in_help,
    discard_spare_arguments,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
pub async fn pong(ctx: Context<'_>) -> CommandResult {
    ctx.log_command().await;

    let ping = ctx.ping().await.as_millis();
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
#[poise::command(
    prefix_command,
    slash_command,
    discard_spare_arguments,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
pub async fn pfp(
    ctx: Context<'_>,
    #[description = "the user to display the profile picture of - defaults to you"] user: Option<
        User,
    >,
    #[flag]
    #[description = "show the user's global profile picture, ignoring if they have a server one set"]
    global: bool,
) -> CommandResult {
    ctx.log_command().await;

    if ctx.defer().await.is_err() {
        error!("failed to defer - lag will cause errors!");
    }

    if ctx.guild().is_some() {
        let guild = ctx
            .guild()
            .expect("guild should already be verified")
            .clone();
        let members = guild.members.clone();

        let member = if let Some(user) = user {
            members.get(&user.id).expect("member should exist")
        } else {
            members
                .get(&ctx.author().id)
                .expect("author should be a member")
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
            other_response(member, pfp_type, global)
        };

        let attachment = CreateAttachment::url(ctx.http(), &pfp).await?;

        ctx.send(
            CreateReply::default()
                .content(response_text)
                .attachment(attachment),
        )
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

        ctx.send(
            CreateReply::default()
                .content(response_text)
                .attachment(CreateAttachment::url(ctx.http(), &pfp).await?),
        )
        .await?;
    }

    Ok(())
}

#[instrument(skip(ctx))]
#[poise::command(
    slash_command,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
pub async fn echo(ctx: Context<'_>, channel: Option<Channel>, message: String) -> CommandResult {
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
#[poise::command(prefix_command, required_bot_permissions = "SEND_MESSAGES")]
pub async fn ban(ctx: Context<'_>, user: User, reason: Option<String>) -> CommandResult {
    if ctx.author().id == 497014954935713802 || user.id == 966519580266737715 {
        ban::joke_ban(ctx, ctx.author(), 966519580266737715, "sike".to_string()).await?;
    } else {
        ban::joke_ban(ctx, &user, ctx.author().id.get(), reason).await?;
    }

    Ok(())
}

#[instrument(skip(ctx))]
#[poise::command(prefix_command, required_bot_permissions = "SEND_MESSAGES")]
pub async fn banban(ctx: Context<'_>) -> CommandResult {
    if ctx.author().id == 497014954935713802 {
        ban::joke_ban(
            ctx,
            ctx.author(),
            966519580266737715,
            "get banbanned lol".to_string(),
        )
        .await?;
    } else {
        ctx.send(CreateReply::default().content("https://files.catbox.moe/jm6sr9.png"))
            .await
            .expect("banban image should be valid");
    }

    Ok(())
}

#[instrument(skip(ctx))]
#[poise::command(prefix_command, required_bot_permissions = "SEND_MESSAGES")]
pub async fn uptime(ctx: Context<'_>) -> CommandResult {
    ctx.log_command().await;

    let started = ctx.data().started;
    let uptime = chrono::Utc::now() - started;

    ctx.reply(format!(
        "uptime: {} (since {})",
        uptime.format_full(),
        started.format("%Y-%m-%d %H:%M UTC")
    ))
    .await
    .expect("sending message should not fail");

    Ok(())
}

#[instrument(skip_all)]
#[poise::command(prefix_command)]
pub async fn purge_after(ctx: Context<'_>, id: MessageId) -> CommandResult {
    ctx.log_command().await;

    let messages = ctx.channel_id().messages_iter(ctx.http());

    let targeted = messages.filter_map(|msg| async move {
        if let Ok(msg) = msg {
            if msg.id >= id {
                Some(msg)
            } else {
                None
            }
        } else {
            None
        }
    });

    //println!("{:?}", Box::pin(messages).next().await);

    targeted
        .for_each(|msg| async move {
            msg.delete(ctx.http())
                .await
                .expect("deleting message should not fail");
            info!("deleted message {}: {}", msg.id, msg.content);
        })
        .await;

    info!("done!");

    /*let content = messages.try_fold(
        String::new(),
        |acc, m| async move { Ok(acc + "\n" + &m.content) }
    ).await.unwrap();

    println!("{content}");

    ctx.say(content).await.unwrap();*/

    Ok(())
}

#[instrument(skip_all)]
#[poise::command(
    slash_command,
    prefix_command,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
pub async fn borzoi(ctx: Context<'_>) -> CommandResult {
    ctx.log_command().await;

    #[derive(Deserialize)]
    struct DogApiResponse {
        message: String,
    }

    let response = reqwest::get("https://dog.ceo/api/breed/borzoi/images/random").await?;

    if response.status().is_server_error() {
        ctx.reply("sorry, dog api is down!").await?;

        return Err(Error::Manual(anyhow!("dog api down")));
    }

    let image_url = response.json::<DogApiResponse>().await?.message;

    let attachment = CreateAttachment::url(&ctx, &image_url).await?;

    let reply = ctx.reply_builder(
        CreateReply::default()
            .content("borzoi courtesy of [dog.ceo](<https://dog.ceo/dog-api/>)")
            .attachment(attachment)
            .reply(true),
    );

    ctx.send(reply).await?;

    Ok(())
}

#[instrument(skip_all)]
#[poise::command(
    slash_command,
    prefix_command,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
pub async fn cat(ctx: Context<'_>, #[flag] gif: bool) -> CommandResult {
    ctx.log_command().await;

    let (url, filename) = if gif {
        ("https://cataas.com/cat/gif", "cat.gif")
    } else {
        ("https://cataas.com/cat", "cat.jpg") // i don't know why this works
                                              // but asserting all images, even png ones, as .jpg is... fine, i guess?
                                              // thanks discord
    };

    let response = reqwest::get(url).await?;

    let bytes = response.bytes().await?;

    let attachment = CreateAttachment::bytes(bytes, filename);
    let reply = CreateReply::default()
        .content("cat courtesy of [cataas.com](<https://cataas.com/>)")
        .attachment(attachment)
        .reply(true);

    ctx.send(reply).await?;

    Ok(())
}

#[instrument(skip_all)]
#[poise::command(
    slash_command,
    prefix_command,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
pub async fn fox(ctx: Context<'_>) -> CommandResult {
    ctx.log_command().await;

    #[derive(Deserialize)]
    struct ApiResponse {
        image: String,
    }

    let json: ApiResponse = reqwest::get("https://randomfox.ca/floof/")
        .await?
        .json::<ApiResponse>()
        .await?;

    let attachment = CreateAttachment::url(&ctx, &json.image).await?;
    let reply = CreateReply::default()
        .content("fox courtesy of [randomfox.ca](<https://randomfox.ca/>)")
        .attachment(attachment)
        .reply(true);

    ctx.send(reply).await?;

    Ok(())
}

pub use minecraft::minecraft;

use self::wordle::DailyPuzzles;
mod minecraft {
    use super::{CommandResult, Context, LogCommands};
    use poise::{serenity_prelude::CreateEmbed, CreateReply};
    use serde::Deserialize;
    use tracing::{debug, instrument};

    #[derive(Deserialize, Clone, Debug)]
    struct ApiResponse {
        online: bool,
        version: Option<ApiResponseVersion>,
        players: Option<ApiResponsePlayers>,
    }

    impl ApiResponse {
        fn version(&self) -> &ApiResponseVersion {
            self.version
                .as_ref()
                .expect("online api response should have version")
        }

        fn players(&self) -> &ApiResponsePlayers {
            self.players
                .as_ref()
                .expect("online api response should have players")
        }
    }

    #[derive(Deserialize, Clone, Debug)]
    struct ApiResponseVersion {
        #[serde(rename = "name_clean")]
        name_clean: String,
    }

    #[derive(Deserialize, Clone, Debug)]
    struct ApiResponsePlayers {
        online: u8,
        #[serde(rename = "list")]
        list: Vec<ApiResponsePlayer>,
    }

    #[derive(Deserialize, Clone, Debug)]
    struct ApiResponsePlayer {
        name_clean: String,
    }

    #[instrument(skip_all)]
    #[poise::command(
        slash_command,
        prefix_command,
        required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
    )]
    pub async fn minecraft(ctx: Context<'_>, server: Option<String>) -> CommandResult {
        ctx.log_command().await;

        let address = server.unwrap_or("162.218.211.126".to_owned());
        let request_url = format!("https://api.mcstatus.io/v2/status/java/{address}");

        let response = reqwest::get(request_url)
            .await?
            .json::<ApiResponse>()
            .await?;

        debug!("{:#?}", response);

        let mut embed = CreateEmbed::default();
        embed = embed.title(address);

        if response.online {
            let players_online = response.players().online;
            embed = embed.description(format!("players online: {players_online}"));

            embed = embed.fields(
                response
                    .players()
                    .list
                    .iter()
                    .map(|p| (&p.name_clean, "", false)),
            );
        }

        ctx.send(CreateReply::default().embed(embed)).await?;

        Ok(())
    }
}

#[instrument(skip_all)]
#[poise::command(
    slash_command,
    prefix_command,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
pub async fn roll(ctx: Context<'_>, #[rest] text: String) -> CommandResult {
    let mut roll = DiceRoll::parse(&text)?;
    let roll2 = roll.clone();

    let rolls = roll.rolls();
    let total = roll.total();

    let faces = roll.dice.next().expect("at least one die").faces;

    let total = if faces.get() == 1 || (faces.get() == 2 && rolls.clone().count() == 1) {
        total.to_string()
    } else {
        match total {
            t if t == roll2.clone().min() || t == roll2.clone().max() => format!("__{t}__"),
            other => other.to_string(),
        }
    };

    debug!(total);

    let text = if roll.extra == 0 {
        if roll.dice.len().get() == 1 {
            format!("**{total}**")
        } else {
            #[allow(clippy::collapsible_else_if)]
            let roll_text = if faces.get() > 2 {
                rolls
                    .map(|n| match n.get() {
                        n if n == 1 || n == faces.get() => format!("__{n}__"),
                        _ => n.to_string(),
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            } else {
                rolls.map(|n| n.to_string()).collect::<Vec<_>>().join(", ")
            };

            format!("**{total}** ({roll_text})")
        }
    } else {
        let extra = match roll.extra {
            n if n > 0 => format!(", +{n}"),
            n if n < 0 => format!(", {n}"),
            _ => unreachable!(),
        };

        #[allow(clippy::collapsible_else_if)]
        let roll_text = if faces.get() > 2 {
            rolls
                .map(|n| match n.get() {
                    n if n == 1 || n == faces.get() => format!("__{n}__"),
                    _ => n.to_string(),
                })
                .collect::<Vec<_>>()
                .join(", ")
        } else {
            rolls.map(|n| n.to_string()).collect::<Vec<_>>().join(", ")
        };

        format!("**{total}** ({roll_text}{extra})")
    };

    ctx.reply(text).await?;

    Ok(())
}

#[instrument(skip_all)]
#[poise::command(
    slash_command,
    prefix_command,
    discard_spare_arguments,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
pub async fn d20(ctx: Context<'_>) -> CommandResult {
    let _typing = ctx.defer_or_broadcast().await?;

    let die = Die::d20();
    let rolled = die.roll().get();

    ctx.reply(format!("**{rolled}**")).await?;

    Ok(())
}

#[instrument(skip_all)]
#[poise::command(
    slash_command,
    prefix_command,
    discard_spare_arguments,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
pub async fn flip(ctx: Context<'_>, coins: Option<u8>, #[flag] verbose: bool) -> CommandResult {
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

    ctx.reply(text).await?;

    Ok(())
}

pub mod roll;

#[instrument(skip_all)]
#[poise::command(
    slash_command,
    prefix_command,
    discard_spare_arguments,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
pub async fn version(ctx: Context<'_>) -> CommandResult {
    let build = if built_info::DEBUG {
        let branch = built_info::GIT_HEAD_REF
            .map(|s| s.split('/').last().expect("head ref should have slashes"))
            .unwrap_or("DETACHED");

        format!(
            "development branch {} (`{}`)",
            branch,
            built_info::GIT_COMMIT_HASH_SHORT.expect("should be built with a git repo")
        )
    } else {
        format!("release {}", built_info::PKG_VERSION)
    };

    ctx.reply(build).await?;

    Ok(())
}

pub mod wordle;

#[instrument(skip_all)]
#[poise::command(
    slash_command,
    prefix_command,
    discard_spare_arguments,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL",
    subcommands("display", "daily", "random")
)]
pub async fn wordle(ctx: Context<'_>) -> CommandResult {
    let typing = ctx.defer_or_broadcast().await?;

    let dm = ctx.author().create_dm_channel(&ctx).await?;
    let puzzles = ctx.data().wordle.puzzles();

    let mut playable = puzzles.playable_for(ctx.author().id).await?.peekable();

    let answers = ctx.data().wordle().answers();

    let mut daily_button = CreateButton::new("daily")
        .label("daily")
        .emoji(ReactionType::Unicode("📅".to_owned()))
        .style(poise::serenity_prelude::ButtonStyle::Primary);

    daily_button = if playable.peek().is_some() {
        daily_button
    } else {
        daily_button.disabled(true)
    };

    let menu_text = match playable.count() {
        0 => "you don't have any daily wordles available. play a random game?",
        1 => "you have 1 new daily wordle available!",
        2 => "you have 2 new daily wordles available!",
        _ => unreachable!(),
    };

    let menu = CreateMessage::new()
        .content(menu_text)
        .button(daily_button)
        .button(
            CreateButton::new("random")
                .label("random")
                .emoji(ReactionType::Unicode("🎲".to_owned()))
                .style(poise::serenity_prelude::ButtonStyle::Secondary),
        )
        .button(
            CreateButton::new("cancel")
                .label("cancel")
                .emoji(ReactionType::Unicode("🚫".to_owned()))
                .style(poise::serenity_prelude::ButtonStyle::Secondary),
        );

    let menu = ctx.channel_id().send_message(ctx, menu).await?;

    drop(typing);

    while let Some(clicked) = menu.await_component_interactions(ctx).stream().next().await {
        if clicked.user.id == ctx.author().id {
            match clicked.data.custom_id.as_str() {
                "daily" => {
                    let typing = ctx.defer_or_broadcast().await?;
                    let mut game = wordle_in_dm(ctx).await?;
                    drop(typing);
                    wordle_play(ctx, &mut game, dm.id, Some(menu.clone())).await?;

                    if game.is_daily() {
                        puzzles.completed(game).await?;
                    }
                }
                "random" => {
                    let typing = ctx.defer_or_broadcast().await?;
                    let mut game = Game::random(ctx.author().id, answers);
                    drop(typing);
                    wordle_play(ctx, &mut game, ctx.channel_id(), Some(menu.clone())).await?;
                }
                "cancel" => {
                    clicked
                        .create_response(ctx, CreateInteractionResponse::Acknowledge)
                        .await?;

                    menu.delete(ctx).await?;
                }
                _ => unreachable!(),
            }
        } else {
            clicked
                .create_response(
                    ctx,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .ephemeral(true)
                            .content("you can't manage a game you didn't start!"),
                    ),
                )
                .await?;
        }
    }

    Ok(())
}

#[instrument(skip_all)]
#[poise::command(
    slash_command,
    prefix_command,
    discard_spare_arguments,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
pub async fn display(ctx: Context<'_>, puzzle: u32, user: Option<User>) -> CommandResult {
    let _typing = ctx.defer_or_broadcast().await?;

    let user = user.map_or_else(|| ctx.author().clone(), |user| user);

    let puzzle = ctx
        .data()
        .wordle()
        .puzzles()
        .collection()
        .find_one(doc! { "_id": puzzle }, None)
        .await?
        .ok_or(InputError::Anyhow(anyhow!("that puzzle doesn't exist!")))?;

    if let Some(result) = puzzle.get_completion(user.id) {
        ctx.reply(format!(
            "wordle #{} (`{}`)\n{}",
            puzzle.number,
            user.name,
            result.as_emoji()
        ))
        .await?;
    } else {
        Err(InputError::Anyhow(anyhow!(
            "{} has not completed wordle #{}!",
            user.name,
            puzzle.number
        )))?;
    }

    Ok(())
}

#[instrument(skip_all)]
#[poise::command(
    slash_command,
    prefix_command,
    discard_spare_arguments,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
pub async fn daily(ctx: Context<'_>) -> CommandResult {
    let _typing = ctx.defer_or_broadcast().await?;

    let dm = ctx.author().create_dm_channel(ctx).await?;

    if ctx
        .data()
        .wordle
        .puzzles
        .playable_for(ctx.author().id)
        .await?
        .next()
        .is_none()
    {
        if let Some(latest) = ctx.data().wordle.puzzles.latest().await? {
            if !latest.is_old() {
                ctx.reply(format!(
                    "you don't have a daily wordle ready yet! check back in {} hours",
                    24 - latest.started.age_hours().unwrap()
                ))
                .await?;

                return Ok(());
            }
        }
    }

    if ctx.guild_id().is_some() {
        ctx.reply("daily wordle can't be played in servers - check your dms!")
            .await?;
    }

    let mut game = wordle_in_dm(ctx).await?;
    drop(_typing);
    wordle_play(ctx, &mut game, dm.id, None).await?;

    if game.is_daily() {
        ctx.data().wordle.puzzles.completed(game).await?;
    }

    Ok(())
}

#[instrument(skip_all)]
#[poise::command(
    slash_command,
    prefix_command,
    discard_spare_arguments,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
pub async fn random(ctx: Context<'_>) -> CommandResult {
    let _typing = ctx.defer_or_broadcast().await?;

    let mut game = Game::random(ctx.author().id, ctx.data().wordle.answers());
    drop(_typing);
    wordle_play(ctx, &mut game, ctx.channel_id(), None).await?;

    Ok(())
}

async fn wordle_in_dm(ctx: Context<'_>) -> Result<wordle::Game, Error> {
    let user = ctx.author().id;

    let puzzles = ctx.data().wordle.puzzles();
    let games = ctx.data().wordle.games();
    let words = ctx.data().wordle.answers();

    let mut playable = puzzles.playable_for(user).await?;

    let game = if let Some(puzzle) = playable.next() {
        trace!(puzzle.number);

        if let Some(unfinished) = games.find_uncompleted_daily(user, puzzle.number).await? {
            puzzle.resume(unfinished)
        } else {
            puzzle.play(user)
        }
    } else if let Some(latest) = puzzles.latest().await? {
        if latest.is_old() || latest.is_expired() {
            puzzles.new_puzzle().await?.play(user)
        } else {
            Game::random(user, words)
        }
    } else {
        puzzles.new_puzzle().await?.play(user)
    };

    /*
    let game = if let Some(next) = puzzles.playable_for(user).await?.next() {
        next.play(user)
    } else if let Some(latest) = puzzles.latest().await? {
        if let Some(unfinished) = ctx
            .data()
            .wordle
            .games
            .find_daily(user, latest.number)
            .await?
        {
            latest.resume(unfinished)
        } else if latest.is_old() {
            puzzles.new_puzzle().await?.play(user)
        } else {
            Game::random(user, &puzzles.words)
        }
    } else {
        puzzles.new_puzzle().await?.play(user)
    };*/

    Ok(game)
}

async fn wordle_play(
    ctx: Context<'_>,
    game: &mut Game,
    channel: ChannelId,
    message: Option<Message>,
) -> CommandResult {
    debug!(%game.answer);

    let title = match game.started.is_old() {
        None => "free play".to_owned(),
        Some(true) => format!(
            "wordle #{} (in backlog)",
            game.number.expect("daily game has number")
        ),
        Some(false) => format!("wordle #{}", game.number.expect("daily game has number")),
    };

    let guesses = ctx.data().wordle.guesses();

    let msg = CreateMessage::new()
        .content(format!("{title}\nno guesses yet!"))
        .button(if game.is_daily() {
            CreateButton::new("pause")
                .emoji(ReactionType::Unicode("⏸️".to_owned()))
                .label("pause")
                .style(poise::serenity_prelude::ButtonStyle::Primary)
        } else {
            CreateButton::new("cancel")
                .emoji(ReactionType::Unicode("🚫".to_owned()))
                .label("cancel")
                .style(poise::serenity_prelude::ButtonStyle::Secondary)
        })
        .button(
            CreateButton::new("give_up")
                .emoji(ReactionType::Unicode("🏳️".to_owned()))
                .label("give up")
                .style(poise::serenity_prelude::ButtonStyle::Danger),
        );

    if let Some(old) = message {
        old.delete(ctx).await?;
    }

    let mut game_message = channel.send_message(&ctx, msg).await?;

    let mut replies = channel.await_replies(ctx).stream();
    let mut interactions = game_message.await_component_interactions(ctx).stream();

    loop {
        tokio::select! {
            Some(msg) = replies.next() => {
                let content = msg.content.as_str();

                if content.len() == 5 {
                    if guesses.contains(content) {
                        msg
                            .react(&ctx, ReactionType::Unicode("✅".to_owned()))
                            .await?;

                        game.guess(content);
                        game_message.edit(
                            ctx,
                            EditMessage::new().content(format!(
                                "{title} {}/6\n{}",
                                game.guesses(),
                                game.emoji()
                            )),
                        )
                        .await?;
                        debug!(guess = ?game.last_guess());

                        if game.is_daily() {
                            ctx.data()
                                .wordle
                                .games
                                .save_game(game)
                                .await?
                                .expect("only saving daily games");
                        }

                        if game.solved() {
                            trace!("solved!");
                            msg.reply(ctx, "you win!").await?;
                            break;
                        }
                    } else {
                        msg
                            .react(&ctx, ReactionType::Unicode("❓".to_owned()))
                            .await?;
                    }
                } else if content.len() == 6 {
                    msg
                        .react(&ctx, ReactionType::Unicode("❌".to_owned()))
                        .await?;

                    msg
                        .react(&ctx, ReactionType::Unicode("6️⃣".to_owned()))
                        .await?;
                }
            }

            Some(interaction) = interactions.next() => {
                if interaction.user.id != game.user {
                    interaction.create_response(
                        ctx,
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new()
                                .ephemeral(true)
                                .content("you can't manage a game you didn't start!"),
                        ),
                    )
                    .await?;
                    continue;
                }

                let blank_confirm = CreateInteractionResponseMessage::new()
                    .button(
                        CreateButton::new("yes")
                            .emoji(ReactionType::Unicode("✅".to_owned()))
                            .label("yes")
                            .style(poise::serenity_prelude::ButtonStyle::Secondary),
                    )
                    .button(
                        CreateButton::new("no")
                            .emoji(ReactionType::Unicode("❌".to_owned()))
                            .label("no")
                            .style(poise::serenity_prelude::ButtonStyle::Secondary),
                    ).ephemeral(true);

                match interaction.data.custom_id.as_str() {
                    "cancel" => {
                        let confirm_message = blank_confirm.content("really cancel?");
                        interaction
                            .create_response(ctx, CreateInteractionResponse::Message(confirm_message))
                            .await?;

                        let message = interaction.get_response(ctx).await?;

                        if let Some(response) = message.await_component_interaction(ctx).await {
                            if response.data.custom_id == "yes" {
                                interaction.create_followup(ctx, CreateInteractionResponseFollowup::new().content("canceled!")).await?;
                                break;
                            }
                        }
                    }
                    "give_up" => {
                        let confirm_message = blank_confirm.content("really give up?");
                        interaction
                            .create_response(ctx, CreateInteractionResponse::Message(confirm_message))
                            .await?;

                        let message = interaction.get_response(ctx).await?;

                        if let Some(response) = message.await_component_interaction(ctx).await {
                            if response.data.custom_id == "yes" {
                                game.ended = true;
                                interaction.create_followup(ctx, CreateInteractionResponseFollowup::new().content(format!("the word was: {}", game.answer))).await?;
                                break;
                            }
                        }
                    }
                    "pause" => {
                        interaction.create_response(ctx, CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().content("your game has been saved - resume with `!!wordle daily`"))).await?;
                        break;
                    }
                    _ => unreachable!(),
                }
            }
        }
    }

    Ok(())
}
