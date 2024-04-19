use poise::serenity_prelude::{
    self,
    futures::{Stream, StreamExt},
    CacheHttp, ChannelId, ComponentInteraction, CreateActionRow, CreateButton,
    CreateInteractionResponseMessage, EditMessage, Http, Message, MessageId, ReactionType,
    ShardMessenger, UserId,
};

use crate::{
    functions::games::wordle::{
        core::AsEmoji, utils::ComponentInteractionExt as UtilsComponentInteractionExt,
    },
    Context, WordleData,
};

use super::{
    core::{
        guess::GuessSlice, Guess, Guesses, GuessesRecord, PartialGuess, PartialGuessError,
        ToPartialGuess,
    },
    puzzle::Puzzle,
    DailyWordles, GameState, GameStyle, WordsList,
};

type SerenityResult<T> = serenity_prelude::Result<T>;

pub struct Game<'a> {
    puzzle: Puzzle,
    guesses: Guesses,
    ctx: Context<'a>,
    msg: &'a mut Message,
    words: &'a WordsList,
    dailies: &'a DailyWordles,
    data: &'a WordleData,
    style: GameStyle,
}

impl<'a> Game<'a> {
    pub fn new(
        ctx: Context<'a>,
        msg: &'a mut Message,
        words: &'a WordsList,
        dailies: &'a DailyWordles,
        data: &'a WordleData,
        puzzle: impl Into<Puzzle>,
        style: Option<GameStyle>,
    ) -> Self {
        Self {
            puzzle: puzzle.into(),
            guesses: Guesses::unlimited(),
            ctx,
            msg,
            words,
            dailies,
            data,
            style: style.unwrap_or_default(),
        }
    }

    pub fn channel_id(&self) -> ChannelId {
        *self.as_ref()
    }

    pub fn message_id(&self) -> MessageId {
        *self.as_ref()
    }

    pub async fn setup(&mut self) -> SerenityResult<()> {
        self.update_message().await?;
        self.add_buttons().await
    }

    pub async fn add_buttons(&mut self) -> SerenityResult<()> {
        self.msg
            .edit(
                self.ctx,
                EditMessage::new().components(vec![self.buttons_builder()]),
            )
            .await
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

    pub fn title(&self) -> String {
        format!("{} {}/6", self.puzzle.title(), self.guesses.count())
    }

    pub fn content(&self) -> String {
        format!(
            "{}\n{}",
            self.title(),
            self.guesses.emoji_with_style(self.style)
        )
    }

    pub async fn update_message(&mut self) -> SerenityResult<()> {
        self.msg
            .edit(self.ctx, EditMessage::new().content(self.content()))
            .await
    }

    pub fn puzzle(&self) -> &Puzzle {
        &self.puzzle
    }

    pub fn messages_stream(&self) -> impl Stream<Item = Message> {
        self.msg.channel_id.await_replies(self.ctx).stream()
    }

    pub fn buttons_stream(&self) -> impl Stream<Item = ComponentInteraction> {
        self.msg.await_component_interactions(self.ctx).stream()
    }

    pub fn guess(&mut self, partial: PartialGuess) -> Guess {
        let new = self.puzzle.guess(partial);
        self.guesses.push(new);
        self.guesses.last().expect("just added one")
    }

    pub async fn finish(&mut self, text: impl AsRef<str>) -> SerenityResult<()> {
        let ctx = self.ctx;
        let new_content = format!("{}\n{}", self.content(), text.as_ref());

        self.msg
            .edit(ctx, EditMessage::new().content(new_content))
            .await
    }

    pub fn is_solved(&self) -> bool {
        self.guesses.last_is_solved()
    }

    pub fn state(&self, finished: bool) -> GameState {
        GameState::new(self.author_id(), self.guesses.to_record(), finished)
    }

    pub fn data(&self) -> GameData {
        GameData {
            guesses: self.guesses.to_record(),
            channel_id: self.channel_id(),
            message_id: self.message_id(),
        }
    }

    pub fn buttons_builder(&self) -> CreateActionRow {
        let pause_cancel_button = if self.puzzle().is_daily() {
            CreateButton::new("pause")
                .emoji(ReactionType::Unicode("⏸️".to_owned()))
                .label("pause")
                .style(poise::serenity_prelude::ButtonStyle::Primary)
        } else {
            CreateButton::new("cancel")
                .emoji(ReactionType::Unicode("🚫".to_owned()))
                .label("cancel")
                .style(poise::serenity_prelude::ButtonStyle::Secondary)
        };

        let give_up_button = CreateButton::new("give_up")
            .emoji(ReactionType::Unicode("🏳️".to_owned()))
            .label("give up")
            .style(poise::serenity_prelude::ButtonStyle::Danger);

        let buttons = vec![pause_cancel_button, give_up_button];

        CreateActionRow::Buttons(buttons)
    }

    pub async fn run(&mut self) -> Result<(), crate::errors::CommandError> {
        let ctx = self.context();

        let mut messages = self.messages_stream();
        let mut interactions = self.buttons_stream();

        loop {
            tokio::select! {
                Some(msg) = messages.next() => {
                    if let Some(partial) = msg.find_guess(ctx).await? {
                        self.guess(partial);

                        self.update_message().await?;

                        if let Some(num) = self.puzzle.number() {
                            self.dailies.update(num, self.state(self.is_solved())).await?;
                        }

                        self.data.update_data(self.channel_id(), self.data()).await;

                        if self.is_solved() {
                            msg.reply(ctx, "you win!").await?;
                            break;
                        }
                    }
                },
                Some(interaction) = interactions.next() => {
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

                                self.finish("game over!").await?;
                                break;
                            }
                            _ => unreachable!()
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

impl AsRef<ChannelId> for Game<'_> {
    fn as_ref(&self) -> &ChannelId {
        &self.msg.channel_id
    }
}

impl AsRef<MessageId> for Game<'_> {
    fn as_ref(&self) -> &MessageId {
        &self.msg.id
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

#[derive(Clone, Debug)]
pub struct GameData {
    pub guesses: GuessesRecord,
    pub channel_id: ChannelId,
    pub message_id: MessageId,
}

trait AddButton: Sized + Clone {
    fn add_button(mut self, button: CreateButton) -> Self {
        self.add_button_in_place(button);
        self
    }

    fn add_button_in_place(&mut self, button: CreateButton) {
        let cloned = self.clone();
        *self = cloned.add_button(button);
    }

    fn add_buttons(mut self, buttons: &[CreateButton]) -> Self {
        for button in buttons {
            self = self.add_button(button.clone());
        }

        self
    }

    #[allow(dead_code)]
    fn add_buttons_in_place(&mut self, buttons: &[CreateButton]) {
        for button in buttons {
            self.add_button_in_place(button.clone());
        }
    }
}

impl AddButton for CreateInteractionResponseMessage {
    fn add_button(self, button: CreateButton) -> Self {
        self.button(button)
    }
}

trait YesNoButtons: AddButton {
    fn yes_no_buttons(self) -> Self {
        let yes_emoji = ReactionType::Unicode("✅".to_owned());
        let no_emoji = ReactionType::Unicode("❌".to_owned());

        let yes_button = CreateButton::new("yes")
            .emoji(yes_emoji)
            .label("yes")
            .style(poise::serenity_prelude::ButtonStyle::Secondary);

        let no_button = CreateButton::new("no")
            .emoji(no_emoji)
            .label("no")
            .style(poise::serenity_prelude::ButtonStyle::Secondary);

        self.add_buttons(&[yes_button, no_button])
    }
}

impl<T> YesNoButtons for T where T: AddButton {}
