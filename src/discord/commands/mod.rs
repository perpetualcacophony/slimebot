mod ban;
mod watch_fic;

use std::{num::{NonZeroI8, NonZeroU8}, ops::Neg, str::FromStr};

use anyhow::anyhow;
use poise::{
    serenity_prelude::{
        futures::StreamExt, CacheHttp, Channel, CreateAttachment, Member, MessageId, User,
    },
    CreateReply,
};
use rand::seq::IteratorRandom;
use regex::Regex;
use serde::Deserialize;

#[allow(unused_imports)]
use tracing::{debug, error, info, instrument};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, crate::Data, Error>;

type CommandResult = Result<(), Error>;

pub use watch_fic::watch_fic;

use crate::{discord::commands::roll::{DiceRoll, NaturalI8}, FormatDuration};

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
#[poise::command(slash_command, prefix_command, discard_spare_arguments)]
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
#[poise::command(slash_command, prefix_command, hide_in_help, discard_spare_arguments)]
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
#[poise::command(prefix_command, slash_command, discard_spare_arguments)]
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
#[poise::command(slash_command)]
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
#[poise::command(prefix_command)]
pub async fn ban(ctx: Context<'_>, user: User, reason: Option<String>) -> CommandResult {
    if ctx.author().id == 497014954935713802 || user.id == 966519580266737715 {
        ban::joke_ban(ctx, ctx.author(), 966519580266737715, "sike".to_string()).await?;
    } else {
        ban::joke_ban(ctx, &user, ctx.author().id.get(), reason).await?;
    }

    Ok(())
}

#[instrument(skip(ctx))]
#[poise::command(prefix_command)]
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
#[poise::command(prefix_command)]
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
#[poise::command(slash_command, prefix_command)]
pub async fn borzoi(ctx: Context<'_>) -> CommandResult {
    ctx.log_command().await;

    #[derive(Deserialize)]
    struct DogApiResponse {
        message: String,
    }

    let response = reqwest::get("https://dog.ceo/api/breed/borzoi/images/random").await?;

    if response.status().is_server_error() {
        ctx.reply("sorry, dog api is down!")
            .await
            .expect("sending message should not fail");
        return Err(anyhow!("dog api down").into());
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
#[poise::command(slash_command, prefix_command)]
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
#[poise::command(slash_command, prefix_command)]
pub async fn fox(ctx: Context<'_>) -> CommandResult {
    ctx.log_command().await;

    #[derive(Deserialize)]
    struct ApiResponse {
        image: String,
    }
    
    let json: ApiResponse = reqwest::get("https://randomfox.ca/floof/").await?
        .json::<ApiResponse>().await?;

    let attachment = CreateAttachment::url(&ctx, &json.image).await?;
    let reply = CreateReply::default()
        .content("fox courtesy of [randomfox.ca](<https://randomfox.ca/>)")
        .attachment(attachment)
        .reply(true);

    ctx.send(reply).await?;

    Ok(())
}


pub use minecraft::minecraft;
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
    #[poise::command(slash_command, prefix_command)]
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
#[poise::command(slash_command, prefix_command)]
pub async fn roll(ctx: Context<'_>, #[rest] text: String) -> CommandResult {
    let mut roll = DiceRoll::parse(&text).unwrap();
    let mut roll2 = roll.clone();

    let rolls = roll.rolls();
    let total = roll.total();

    let faces = roll.dice.next().unwrap().faces;

    let total = if faces.get() == 1 || (faces.get() == 2 && rolls.clone().count() == 1) {
        total.to_string()
    } else {
        match total {
            t if t == roll2.clone().min() || t == roll2.clone().max() => format!("__{t}__"),
            other => other.to_string()
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
                .map(|n| {
                    match n.get() {
                        n if n == 1 || n == faces.get() => format!("__{n}__"),
                        _ => n.to_string()
                    }
                })
                .collect::<Vec<_>>()
                .join(", ")
            } else {
                rolls
                .map(|n| {
                    n.to_string()
                })
                .collect::<Vec<_>>()
                .join(", ")
            };

            format!("**{total}** ({roll_text})")
        }
    } else {
        let extra = match roll.extra {
            n if n > 0 => format!(", +{n}"),
            n if n < 0 => format!(", {n}"),
            _ => unreachable!()
        };

        #[allow(clippy::collapsible_else_if)]
        let roll_text = if faces.get() > 2 {
            rolls
            .map(|n| {
                match n.get() {
                    n if n == 1 || n == faces.get() => format!("__{n}__"),
                    _ => n.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join(", ")
        } else {
            rolls
            .map(|n| {
                n.to_string()
            })
            .collect::<Vec<_>>()
            .join(", ")
        };

        format!("**{total}** ({roll_text}{extra})")
    };

    ctx.reply(text).await?;

    Ok(())
}

#[instrument(skip_all)]
#[poise::command(slash_command, prefix_command, discard_spare_arguments)]
pub async fn flip(ctx: Context<'_>, coins: u8)-> CommandResult {
    //let roll = DiceRoll::parse(&text).unwrap();

    //let rolled = roll.execute();

    //ctx.reply(rolled.to_string()).await.unwrap();

    Ok(())
}

mod roll {
    use std::{default, iter::Sum, num::{NonZeroI8, ParseIntError, TryFromIntError}, ops::Neg, str::FromStr};

    use mongodb::bson::raw::Iter;
    use rand::{rngs::{StdRng, ThreadRng}, seq::IteratorRandom, Rng, SeedableRng};
    use regex::Regex;
    use serde::Deserialize;
    use thiserror::Error;
    use tracing::{debug, instrument, trace};

    #[derive(Debug, Error, PartialEq)]
    pub enum DiceRollError {
        #[error(transparent)]
        InvalidNumber(#[from] NaturalI8Error),
        #[error("")]
        NoFaces,
        #[error("")]
        InvalidExtra(String),
        #[error("")]
        InvalidExtraSign(String),
    }

    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    pub struct Die {
        pub faces: NaturalI8,
    }

    impl Die {
        fn new(faces: NaturalI8) -> Self {
            Self {
                faces,
            }
        }

        fn roll(&self) -> NaturalI8 {
            self.roll_with(&mut rand::thread_rng())
        }

        fn roll_with(&self, rng: &mut impl Rng) -> NaturalI8 {
            let range = 1..=self.faces.get();
            let roll = range.choose(rng)
                .expect("should have at least one face")
                .try_into()
                .expect("faces is a valid NaturalI8");
            roll
        }

        fn d20() -> Self {
            Self::new(NaturalI8::twenty())
        }

        fn min(&self) -> NaturalI8 {
            NaturalI8::min()
        }

        pub fn max(&self) -> NaturalI8 {
            self.faces
        }
    }

    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct Dice {
        vec: Vec<Die>,
        index: usize,
    }

    impl Dice {
        pub fn new(count: NaturalI8, faces: NaturalI8) -> Self {
            let vec = vec![Die::new(faces); count.into()];

            Self {
                vec,
                index: 0
            }
        }

        pub fn roll(&self, rng: StdRng) -> Roll<Self> {
            Roll::new(self.clone(), rng)
        }

        pub fn len(&self) -> NaturalI8 {
            ExactSizeIterator::len(self).try_into().expect("number of dice should not be 0")
        }

        #[instrument]
        pub fn lowest_roll(&self) -> NaturalI8 {
            debug!(len = ?self.len());
            self.len()
        }

        #[instrument]
        pub fn highest_roll(&self) -> NaturalI8 {
            let highest = self.clone().fold(0, |sum, die| {
                sum + die.max().get()
            });

            debug!(highest);

            highest.try_into().expect("number of dice != 0 and dice.min() == 1")
        }
    }

    impl TryFrom<usize> for NaturalI8 {
        type Error = NaturalI8Error;

        fn try_from(value: usize) -> Result<Self, Self::Error> {
            let int: i8 = value.try_into()?;
            let non_zero: NonZeroI8 = int.try_into()?;

            non_zero.try_into()
        }
    }

    impl Iterator for Dice {
        type Item = Die;

        fn next(&mut self) -> Option<Self::Item> {
            let item: Option<&Die> = self.vec.get(self.index);
            self.index += 1;
            item.copied()
        }
    }

    impl ExactSizeIterator for Dice {
        fn len(&self) -> usize {
            self.vec.len()
        }
    }

    #[derive(Clone, Debug, PartialEq)]
    pub struct Roll<It: Iterator<Item = Die>> {
        iter: It,
        rng: StdRng,
    }

    impl<It: Iterator<Item = Die>> Roll<It> {
        fn new(iter: It, rng: StdRng) -> Self {
            Self { iter, rng }
        }

        fn total(self, extra: i8) -> i8 {
            self.sum::<i8>() + extra
        }
    }

    impl<It: Iterator<Item = Die>> Iterator for Roll<It> {
        type Item = NaturalI8;

        fn next(&mut self) -> Option<Self::Item> {
            let die = self.iter.next();
            die.map(|die| die.roll_with(&mut self.rng))
        }
    }

    #[derive(Clone, Debug, PartialEq)]
    pub struct DiceRoll {
        pub dice: Dice,
        pub extra: i8,
        rng: StdRng
    }
    
    impl DiceRoll {
        pub fn new(count: i8, faces: i8, extra: i8) -> Result<Self, DiceRollError> {
            let faces = faces.try_into()?;
            let count = count.try_into()?;

            let dice = Dice::new(count, faces);

            let seed: [u8; 32] = rand::random();
            let rng = StdRng::from_seed(seed);

            let new = Self { dice, extra, rng };
            Ok(new)
        }

        pub fn rolls(&self) -> Roll<Dice> {
            self.dice.roll(self.rng.clone())
        }

        pub fn total(&self) -> i8 {
            let sum = self.rolls().sum::<NaturalI8>();
            sum.get() + self.extra
        }

        #[instrument]
        pub fn parse(text: &str) -> Result<Self, DiceRollError> {
            let regex = Regex::new(r"([0-9]*)d([0-9]+)\s*(?:(\+|-)\s*([0-9]+))?")
                .expect("hard-coded regex should be valid");
    
            let roll = regex.captures(text)
                .map(|caps| {
                    trace!(?caps);

                    let count = caps.get(1)
                        .map_or(Ok(NaturalI8::default()), |mat| mat.as_str().parse())
                        .unwrap_or_default();
                    trace!(?count);
                    let faces: NaturalI8 = caps.get(2).ok_or(DiceRollError::NoFaces)?.as_str().parse()?;
                    trace!(?faces);
    
                    let extra_unsigned = caps.get(4)
                        .map(|mat| {
                            let int = mat.as_str()
                                .parse::<i8>()
                                .map_err(|_| DiceRollError::InvalidExtra(mat.as_str().to_owned()));

                            int.unwrap_or_default()    
                        });
                    trace!(?extra_unsigned);

                    let extra_sign = caps.get(3).map_or("", |mat| mat.as_str());

                    let extra = match extra_sign {
                        "+" => extra_unsigned,
                        "-" => extra_unsigned.map(|int|int.neg()),
                        _ => None,
                    }.unwrap_or_default();
                    debug!(?extra);
    
                    DiceRoll::new(count.get(), faces.get(), extra)
                })
                .expect("");

            debug!(?roll);

            roll

        }

        pub fn min(&self) -> i8 {
            self.dice.lowest_roll().get() + self.extra
        }

        pub fn max(&self) -> i8 {
            self.dice.highest_roll().get() + self.extra
        }
    }
    
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
    pub struct NaturalI8(NonZeroI8);

    macro_rules! natural_const {
        ($name:ident: $num:expr$(,)?) => {
            pub fn $name() -> NaturalI8 {
                NaturalI8::new(
                    NonZeroI8::new(1).expect(format!("{} != 0", $num).as_str())
                ).expect(format!("{} >= 1", $num).as_str())
            }
        };

        ($name:ident: $num:expr, $($names:ident: $nums:expr),+$(,)?) => {
            natural_const!($name: $num);
            natural_const! { $($names: $nums),+ }
        };
    }
    
    impl NaturalI8 {
        natural_const! {
            one: 1,
            twenty: 20,
            one_hundred: 100,
        }

        pub fn new(value: NonZeroI8) -> Result<Self, NaturalI8Error> {
            value.try_into()
        }

        pub fn get(&self) -> i8 {
            self.get_non_zero().get()
        }

        pub fn get_non_zero(&self) -> NonZeroI8 {
            self.0
        }

        pub fn min() -> Self {
            Self::one()
        }
    }

    impl std::iter::Sum for NaturalI8 {
        fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
            iter.map(|natural| natural.get())
                .sum::<i8>()
                .try_into()
                .expect("sum of naturals must be natural")
        }
    }

    impl Default for NaturalI8 {
        fn default() -> Self {
            Self::min()
        }
    }

    impl std::fmt::Debug for NaturalI8 {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    
    impl TryFrom<NonZeroI8> for NaturalI8 {
        type Error = NaturalI8Error;
    
        fn try_from(value: NonZeroI8) -> Result<Self, Self::Error> {
            if value.get() >= 1 {
                Ok(Self(value))
            } else {
                Err(NaturalI8Error::ValueNegative(value))
            }
        }
    }
    
    impl TryFrom<i8> for NaturalI8 {
        type Error = NaturalI8Error;
    
        fn try_from(value: i8) -> Result<Self, Self::Error> {
            let non_zero: NonZeroI8 = value.try_into()?;
    
            if non_zero.get() >= 1 {
                Ok(Self(non_zero))
            } else {
                Err(NaturalI8Error::ValueNegative(non_zero))
            }
        }
    }

    impl Sum<NaturalI8> for i8 {
        fn sum<I: Iterator<Item = NaturalI8>>(iter: I) -> Self {
            iter.map(|natural| natural.get())
                .sum()
        }
    }
    
    impl FromStr for NaturalI8 {
        type Err = NaturalI8Error;
    
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let non_zero: NonZeroI8 = s.parse()?;
    
            if non_zero.get() >= 1 {
                Ok(Self(non_zero))
            } else {
                Err(NaturalI8Error::ValueNegative(non_zero))
            }
        }
    }

    impl From<NaturalI8> for usize {
        fn from(val: NaturalI8) -> Self {
            val.get().try_into().expect("usize > 0 && NaturalI8 > 0")
        }
    }

    impl ToString for NaturalI8 {
        fn to_string(&self) -> String {
            self.get().to_string()
        }
    }

    #[derive(Error, Debug, PartialEq)]
    pub enum NaturalI8Error {
        #[error("parsed value as zero")]
        ParsedZero(#[from] ParseIntError),
        #[error("value cannot be zero")]
        TryFromZero(#[from] TryFromIntError),
        #[error("value `{0}` is negative")]
        ValueNegative(NonZeroI8)
    }

    mod tests {
        use tracing::trace;
        use tracing_test::traced_test;

        use crate::discord::commands::roll::NaturalI8;

        use super::{DiceRoll, Die};

        macro_rules! test_parse {
            ($name:ident: $text:expr => $parsed:expr$(,)?) => {
                #[test]
                fn $name() {
                    pretty_assertions::assert_eq!(
                        super::DiceRoll::parse($text),
                        $parsed
                    )
                }
            };

            ($name:ident: $text:expr => $parsed:expr, $($names:ident: $texts:expr => $parseds:expr),+$(,)?) => {
                test_parse!($name: $text => $parsed);
                test_parse! { $($names: $texts => $parseds),+ }
            };
        }

        test_parse!{
            two_d_ten: "2d10" => DiceRoll::new(2, 10, 0),
            d_twenty: "d20" => DiceRoll::new(1, 20, 0),
            d_six_plus_three: "d6+3" => DiceRoll::new(1, 6, 3),
            two_d_four_minus_two: "2d4-2" => DiceRoll::new(2, 4, -2)
        }

        #[test]
        fn roll_die() {
            let mut die = Die::d20();
            let range = NaturalI8::one()..=NaturalI8::one_hundred();
            let mut rng = rand::thread_rng();

            for _ in 1..1000 {
                let rolled: NaturalI8 = die.roll_with(&mut rng);
                assert!(range.contains(&rolled))
            }
        }

        #[test]
        #[traced_test]
        fn rolls_sensible() {
            let roll = DiceRoll::parse("2d20").expect("hard-coded");
            let range = 2..=40;

            for _ in 1..1000 {
                let rolls = roll.rolls();
                let sum: i8 = rolls.clone().sum();
                trace!(sum, ?rolls);
                assert!(range.contains(&sum))
            }
        }

        #[test]
        #[traced_test]
        fn rolls_sum_sensible() {
            let roll = DiceRoll::parse("2d20+4").expect("hard-coded");
            let range = 6..=44;
            let extra = roll.extra;
            
            for _ in 1..2 {
                let mut roll = roll.clone();
                let sum: i8 = roll.total();
                let rolls = roll.rolls();
                trace!(sum, ?rolls, extra);
                assert!(range.contains(&sum))
            }
        }
    }
}
