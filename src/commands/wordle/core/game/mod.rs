use std::sync::Arc;

use poise::serenity_prelude::{
    self, futures::StreamExt, CacheHttp, ChannelId, ComponentInteraction,
    CreateInteractionResponseMessage, Http, Message, MessageId, ReactionType, ShardMessenger,
    UserId,
};

use crate::{
    utils::{
        poise::ContextExt,
        serenity::{
            buttons::YesNoButtons,
            component_interaction::ComponentInteractionExt as UtilsComponentInteractionExt,
        },
    },
    Context,
};

use self::{message::GameMessage, options::GameOptions};

use super::{
    core::{
        guess::GuessSlice, AsEmoji, Guess, Guesses, PartialGuess, PartialGuessError, ToPartialGuess,
    },
    puzzle::Puzzle,
    DailyWordles, WordsList,
};

type SerenityResult<T> = serenity_prelude::Result<T>;

mod cache;
pub use cache::GamesCache;

mod data;
pub use data::GameData;

mod record;
pub use record::GameRecord;

mod users;
use users::Users;

pub mod options;
use options::GameStyle;

mod message;

pub struct Game<'a> {
    puzzle: Arc<Puzzle>,
    guesses: Guesses,
    ctx: Context<'a>,
    msg: GameMessage,
    words: &'a WordsList,
    dailies: &'a DailyWordles,
    cache: &'a GamesCache,
    users: Users<'a>,
    style: GameStyle,
}

impl<'a> Game<'a> {
    pub async fn new(
        ctx: Context<'a>,
        puzzle: impl Into<Puzzle>,
        options: GameOptions,
    ) -> serenity_prelude::Result<Self> {
        let users = if ctx.in_guild() {
            Users::more(ctx.author())
        } else {
            Users::one(ctx.author())
        };

        let data = ctx.data();
        let puzzle = puzzle.into();
        let msg = GameMessage::new(ctx, &puzzle).await?;

        Ok(Self {
            puzzle: Arc::new(puzzle),
            guesses: Guesses::new(options.guesses_limit),
            ctx,
            msg,
            words: data.wordle().words(),
            dailies: data.wordle().wordles(),
            cache: data.wordle().game_data(),
            users,
            style: options.style,
        })
    }

    pub fn channel_id(&self) -> ChannelId {
        *self.as_ref()
    }

    pub fn message_id(&self) -> MessageId {
        *self.as_ref()
    }

    pub async fn lock_channel(&self) -> Arc<GameData> {
        self.update_data().await
    }

    pub async fn unlock_channel(&self) {
        self.cache.unlock_channel(self.channel_id()).await
    }

    pub async fn update_data(&self) -> Arc<GameData> {
        self.cache.set(self.channel_id(), self.data()).await;
        self.cache.get(self.channel_id()).await.expect("just added")
    }

    pub async fn setup(&mut self) -> SerenityResult<()> {
        let arc = self.lock_channel().await;
        self.msg.edit(self.ctx, arc).await?;
        Ok(())
    }

    fn context(&self) -> GameContext<'a> {
        GameContext {
            poise: self.ctx,
            words_list: self.words,
        }
    }

    pub fn author_id(&self) -> UserId {
        self.ctx.author().id
    }

    pub fn puzzle(&self) -> Arc<Puzzle> {
        self.puzzle.clone()
    }

    pub fn guess(&mut self, partial: PartialGuess) -> Guess {
        let new = self.puzzle.guess(partial);
        self.guesses.push(new);
        self.guesses.last().expect("just added one")
    }

    // pub async fn finish(&mut self, text: impl AsRef<str>) -> SerenityResult<()> {
    //     let ctx = self.ctx;
    //     let new_content = format!("{}\n{}", self.content(), text.as_ref());

    //     self.msg
    //         .edit(ctx, EditMessage::new().content(new_content))
    //         .await
    // }

    pub fn is_solved(&self) -> bool {
        self.guesses.last_is_solved()
    }

    pub fn state(&self, finished: bool) -> GameRecord {
        GameRecord::new(self.author_id(), self.guesses.to_record(), finished)
    }

    pub fn data(&self) -> GameData {
        GameData {
            puzzle: self.puzzle(),
            guesses: self.guesses.clone(),
            channel_id: self.channel_id(),
            message_id: self.message_id(),
        }
    }
    pub async fn run(&mut self) -> Result<(), crate::errors::CommandError> {
        let ctx = self.context();

        let mut messages = self.msg.replies_stream(ctx);
        let mut interactions = self.msg.buttons_stream(ctx);

        loop {
            tokio::select! {
                Some(msg) = messages.next() => {
                    if let Some(partial) = msg.find_guess(ctx).await? {
                        self.guess(partial);

                        let data = self.cache.set(*self.msg.channel_id(), self.data()).await;
                        self.msg.edit(ctx, data).await?;

                        if let Some(num) = self.puzzle.number() {
                            self.dailies.update(num, self.state(self.is_solved())).await?;
                        }

                        if self.is_solved() {
                            msg.reply(ctx, "you win!").await?;
                            break;
                        }

                        if !self.users.contains(&msg.author) {
                            self.users.add(msg.author)
                        }
                    }
                },
                Some(interaction) = interactions.next() => {
                    match interaction.custom_id() {
                        "unused" => {
                            interaction.reply_ephemeral(ctx, format!("unused letters: {}", self.guesses.unused_letters().as_emoji())).await?;
                        }
                        _ => {
                            if interaction.confirmed(ctx).await? {
                                match interaction.custom_id() {
                                    "pause" => {
                                        let number = self.puzzle().number().expect("this option is only available for daily puzzles");
                                        self.dailies.update(number, self.state(false)).await?;
                                        break;
                                    }
                                    "cancel" => {
                                        break;
                                    }
                                    "give_up" => {
                                        if let Some(num) = self.puzzle().number() {
                                            self.dailies.update(num, self.state(true)).await?;
                                        }

                                        self.msg.reply(ctx, format!("the word was: {word}", word = self.puzzle.answer())).await?;

                                        self.msg.finish(ctx, "game over!").await?;
                                        break;
                                    },
                                    _ => unreachable!()
                                }
                            }
                        }
                    }
                }
            }
        }

        self.unlock_channel().await;

        Ok(())
    }
}

impl AsRef<ChannelId> for Game<'_> {
    fn as_ref(&self) -> &ChannelId {
        self.msg.channel_id()
    }
}

impl AsRef<MessageId> for Game<'_> {
    fn as_ref(&self) -> &MessageId {
        self.msg.message_id()
    }
}

trait MessageExt {
    async fn find_guess(
        &self,
        ctx: GameContext<'_>,
    ) -> serenity_prelude::Result<Option<PartialGuess>>;
}

impl MessageExt for Message {
    async fn find_guess(
        &self,
        ctx: GameContext<'_>,
    ) -> serenity_prelude::Result<Option<PartialGuess>> {
        let question_mark: ReactionType = ReactionType::Unicode("❓".to_owned());
        let check_mark: ReactionType = ReactionType::Unicode("✅".to_owned());

        match self.content.to_partial_guess(ctx.words()) {
            Ok(partial) => {
                self.react(ctx, check_mark).await?;
                Ok(Some(partial))
            }
            Err(err) => match err {
                PartialGuessError::NotInList(..) => {
                    self.react(ctx, question_mark).await?;
                    Ok(None)
                }
                _ => Ok(None),
            },
        }
    }
}

trait ComponentInteractionExt {
    async fn confirmed(&self, ctx: GameContext) -> serenity_prelude::Result<bool>;

    async fn await_yes_no(
        &self,
        shard_cache_http: impl AsRef<Http> + AsRef<ShardMessenger> + CacheHttp + Copy,
    ) -> serenity_prelude::Result<Option<bool>>;
}

impl ComponentInteractionExt for ComponentInteraction {
    async fn confirmed(&self, ctx: GameContext<'_>) -> serenity_prelude::Result<bool> {
        if self.user.id != ctx.user_id() {
            self.reply_ephemeral(ctx, "you can only manage a game you started!")
                .await?;
            return Ok(false);
        }

        let action = match self.custom_id() {
            "cancel" => "cancel",
            "give_up" => "give up",
            "pause" => "pause",
            _ => unreachable!(),
        };

        let builder = CreateInteractionResponseMessage::new()
            .content(format!("really {action}?"))
            .ephemeral(true)
            .yes_no_buttons();

        self.respond(ctx, builder).await?;

        //self.delete_response(ctx).await?;

        self.await_yes_no(ctx)
            .await
            .map(|op| op.unwrap_or_default())
    }

    async fn await_yes_no(
        &self,
        shard_cache_http: impl AsRef<Http> + AsRef<ShardMessenger> + CacheHttp + Copy,
    ) -> serenity_prelude::Result<Option<bool>> {
        if let Some(interaction) = self
            .get_response(shard_cache_http)
            .await?
            .await_component_interaction(shard_cache_http)
            .await
        {
            interaction.acknowledge(shard_cache_http).await?;

            let result = match interaction.custom_id() {
                "yes" => Some(true),
                "no" => Some(false),
                _ => None,
            };

            Ok(result)
        } else {
            Ok(None)
        }
    }
}

#[derive(Copy, Clone)]
struct GameContext<'a> {
    poise: Context<'a>,
    words_list: &'a WordsList,
}

impl<'a> GameContext<'a> {
    fn poise_context(&self) -> Context<'a> {
        self.poise
    }

    fn user_id(&self) -> UserId {
        self.poise_context().author().id
    }

    fn words(&self) -> &WordsList {
        self.words_list
    }
}

impl CacheHttp for GameContext<'_> {
    fn http(&self) -> &serenity_prelude::Http {
        self.as_ref()
    }

    fn cache(&self) -> Option<&std::sync::Arc<serenity_prelude::Cache>> {
        self.poise_context().serenity_context().cache()
    }
}

impl AsRef<Http> for GameContext<'_> {
    fn as_ref(&self) -> &Http {
        self.poise_context().http()
    }
}

impl AsRef<ShardMessenger> for GameContext<'_> {
    fn as_ref(&self) -> &ShardMessenger {
        &self.poise_context().serenity_context().shard
    }
}

impl AsRef<WordsList> for GameContext<'_> {
    fn as_ref(&self) -> &WordsList {
        self.words()
    }
}
