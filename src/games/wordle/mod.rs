use std::{
    borrow::Cow,
    collections::HashMap,
    fs,
    future::Future,
    ops::{Index, IndexMut, Not},
    path::Path,
    slice::Iter,
    str::FromStr,
};

use anyhow::anyhow;
use chrono::Utc;
use mongodb::{
    bson::doc,
    options::{FindOneOptions, FindOptions},
    Collection, Database,
};
use poise::{
    serenity_prelude::{
        futures::{Stream, StreamExt, TryFutureExt, TryStreamExt},
        ButtonStyle, CacheHttp, ChannelId, ComponentInteraction, CreateActionRow, CreateButton,
        CreateInteractionResponse, CreateInteractionResponseFollowup,
        CreateInteractionResponseMessage, CreateMessage, EditMessage, Http, Interaction, Message,
        ReactionType, ShardMessenger, UserId,
    },
    Context, CreateReply,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, instrument, trace};

use crate::{config::WordleConfig, UtcDateTime};

const PUZZLE_ACTIVE_HOURS: i64 = 24;

mod error;
pub use error::Error;

mod core;
use core::{AsEmoji, Guess, Word};

use mongodb::error::Error as MongoDbError;

mod puzzle;
use puzzle::Puzzle;

type DbResult<T> = std::result::Result<T, MongoDbError>;
type Result<T> = std::result::Result<T, crate::errors::Error>;

/*
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DailyPuzzle {
    #[serde(rename = "_id")]
    pub number: u32,
    pub started: StartTime,
    answer: String,
    finished: Vec<WordleResults>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct WordleResults {
    user: UserId,
    guesses: Vec<Guess>,
    num_guesses: usize,
    solved: bool,
    ended: bool,
}

impl DailyPuzzle {
    fn new(words: &WordsList, number: u32) -> Self {
        let word = words.get_random().to_owned();

        Self {
            number,
            started: StartTime::now(),
            answer: word,
            ..Default::default()
        }
    }

    pub fn play(&self, user: UserId) -> Game {
        Game {
            user,
            guesses: Vec::with_capacity(6),
            answer: Word::new(&self.answer),
            started: self.started,
            number: Some(self.number),
            ended: false,
        }
    }

    pub fn resume(&self, result: GameResult) -> Game {
        Game {
            user: result.user,
            guesses: result.guesses,
            answer: Word::new(&self.answer),
            started: self.started,
            number: Some(result.puzzle),
            ended: false,
        }
    }

    pub fn completed_by(&self, user: UserId) -> bool {
        self.finished.iter().any(|game| game.user == user)
    }

    pub fn get_completion(&self, user: UserId) -> Option<&GameResult> {
        self.finished.iter().find(|result| result.user == user)
    }

    fn completed(&mut self, completion: Game) {
        //assert!(completion.solved(), "game should be completed");
        assert!(
            completion.answer == self.answer,
            "completion should have the same answer"
        );

        let results = completion.results(true);

        self.finished.push(results);
    }

    #[instrument(skip(self), fields(num = self.number))]
    pub fn is_old(&self) -> bool {
        self.started.is_old().map_or(false, |b| b) && !self.is_expired()
    }

    #[instrument(skip(self), fields(num = self.number))]
    pub fn is_expired(&self) -> bool {
        self.started.is_expired().map_or(false, |b| b)
    }

    #[instrument(skip(self), fields(num = self.number))]
    pub fn is_playable(&self, user: UserId) -> bool {
        debug!(
            expired = self.is_expired(),
            completed = self.completed_by(user)
        );

        !self.is_expired() && !self.completed_by(user)
    }

    #[instrument(skip_all)]
    pub fn is_backlogged(&self, user: UserId) -> bool {
        debug!(playable = self.is_playable(user), old = self.is_old());

        self.is_playable(user) && self.is_old()
    }
}

impl PartialEq<String> for Word {
    fn eq(&self, other: &String) -> bool {
        &self.to_string() == other
    }
}

#[derive(Debug, Clone, Default)]
pub struct WordsList {
    words: Vec<String>,
}

impl WordsList {
    pub fn answers() -> Self {
        let file = fs::read_to_string("./wordle_answers.txt").unwrap_or_else(|_| {
            fs::read_to_string("/wordle_answers.txt")
                .expect("words should be at ./wordle_answers.txt or /wordle_answers.txt")
        });

        let words = file.lines().map(|s| s.to_owned()).collect::<Vec<String>>();

        Self { words }
    }

    pub fn guesses() -> Self {
        let file = fs::read_to_string("./wordle_guesses.txt").unwrap_or_else(|_| {
            fs::read_to_string("/wordle_guesses.txt")
                .expect("words should be at ./wordle_guesses.txt or /wordle_guesses.txt")
        });

        let mut words = file.lines().map(|s| s.to_owned()).collect::<Vec<String>>();

        let other = &mut WordsList::answers().words;

        words.append(other);

        Self { words }
    }

    pub fn contains(&self, word: &str) -> bool {
        self.words.contains(&word.to_lowercase())
    }

    fn get_random(&self) -> &str {
        use rand::prelude::SliceRandom;

        self.words
            .choose(&mut rand::thread_rng())
            .expect("words list should not be empty")
    }
}

#[derive(Debug, Clone)]
pub struct DailyGames {
    collection: Collection<GameResult>,
}

impl DailyGames {
    pub fn get(db: &Database) -> Self {
        let collection = db.collection("wordle_daily_games");
        Self { collection }
    }

    fn collection(&self) -> &Collection<GameResult> {
        &self.collection
    }

    pub async fn find_daily(&self, user: UserId, puzzle: u32) -> DbResult<Option<GameResult>> {
        self.collection()
            .find_one(doc! { "user": user.to_string(), "puzzle": puzzle }, None)
            .await
    }

    pub async fn save_game(&self, game: &Game) -> DbResult<Result<()>> {
        let number = if let Some(n) = game.number {
            n
        } else {
            return Ok(Err(anyhow!("test").into()));
        };

        if let Some(daily) = self.find_daily(game.user, number).await? {
            self.collection()
                .delete_one(
                    doc! { "user": daily.user.to_string(), "puzzle": daily.puzzle },
                    None,
                )
                .await?;
        }

        self.collection()
            .insert_one(game.results(game.solved()), None)
            .await?;

        Ok(Ok(()))
    }

    pub async fn find_uncompleted_daily(
        &self,
        user: UserId,
        puzzle: u32,
    ) -> DbResult<Option<GameResult>> {
        self.collection()
            .find_one(
                doc! { "user": user.to_string(), "puzzle": puzzle, "completed": false },
                None,
            )
            .await
    }
}

#[derive(Debug, Clone)]
pub struct DailyPuzzles {
    collection: Collection<DailyPuzzle>,
    pub answers: WordsList,
}

impl DailyPuzzles {
    pub fn get(db: &Database, words: WordsList) -> Self {
        let collection = db.collection("wordle_daily_puzzles");
        Self {
            collection,
            answers: words,
        }
    }

    pub fn collection(&self) -> &Collection<DailyPuzzle> {
        &self.collection
    }

    pub async fn latest(&self) -> DbResult<Option<DailyPuzzle>> {
        self.collection()
            .find_one(
                None,
                FindOneOptions::builder().sort(doc! { "_id": -1 }).build(),
            )
            .await
    }

    pub async fn new_puzzle(&self) -> DbResult<DailyPuzzle> {
        let latest = self.latest().await?;

        let number = if let Some(latest) = latest {
            latest.number + 1
        } else {
            1
        };

        let puzzle = DailyPuzzle::new(&self.answers, number);

        self.collection().insert_one(&puzzle, None).await?;

        Ok(puzzle)
    }

    pub async fn previous(&self) -> DbResult<Result<DailyPuzzle>> {
        let latest_num = self.latest().await?.map_or(1, |puzzle| puzzle.number);

        Ok(if latest_num == 1 {
            Err(Error::OnlyOnePuzzle)
        } else {
            let previous = self
                .collection()
                .find_one(doc! { "_id": latest_num - 1 }, None)
                .await?
                .expect("more than 1 puzzle, so previous puzzle should exist");

            if !previous.is_expired() {
                Ok(previous)
            } else {
                Err(Error::Expired(previous))
            }
        })
    }

    pub async fn completed(&self, game: Game) -> DbResult<()> {
        let number = game.number.expect("scored game should have number");

        // extremely clunky fix - can't use update functions because of bson limitation
        let puzzle = self
            .collection()
            .find_one(doc! { "_id": number }, None)
            .await?
            .map(|mut puzzle| {
                puzzle.completed(game);
                puzzle
            });

        self.collection()
            .delete_one(doc! { "_id": number }, None)
            .await?;

        if let Some(puzzle) = puzzle {
            self.collection().insert_one(&puzzle, None).await?;
        }

        Ok(())
    }

    #[instrument(skip_all, level = "trace")]
    pub async fn not_expired(&self) -> DbResult<Vec<DailyPuzzle>> {
        let mut cursor = self
            .collection()
            .find(
                None,
                FindOptions::builder()
                    .sort(doc! {"_id":-1})
                    .limit(2)
                    .build(),
            )
            .await?;

        let mut vec = Vec::new();

        while let Some(doc) = cursor.next().await {
            let puzzle = doc?;

            if !puzzle.is_expired() {
                trace!("puzzle {} not expired", puzzle.number);
                vec.push(puzzle)
            } else {
                trace!("puzzle {} expired", puzzle.number);
            }
        }

        Ok(vec)
    }

    pub async fn playable_for(&self, user: UserId) -> DbResult<impl Iterator<Item = DailyPuzzle>> {
        Ok(self
            .not_expired()
            .await?
            .into_iter()
            .rev()
            .filter(move |puzzle| !puzzle.completed_by(user)))
    }
}

#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct StartTime(Option<UtcDateTime>);

impl StartTime {
    fn new(time: UtcDateTime) -> Self {
        Self(Some(time))
    }

    fn now() -> Self {
        Self(Some(Utc::now()))
    }

    fn none() -> Self {
        Self(None)
    }

    pub fn age_hours(&self) -> Option<i64> {
        self.0.map(|start| (Utc::now() - start).num_hours())
    }

    pub fn is_old(&self) -> Option<bool> {
        self.age_hours().map(|age| age >= PUZZLE_ACTIVE_HOURS)
    }

    pub fn is_expired(&self) -> Option<bool> {
        self.age_hours().map(|age| age >= 2 * PUZZLE_ACTIVE_HOURS)
    }
}
*/

use rand::prelude::SliceRandom;

#[derive(Debug, Clone)]
pub struct WordsList {
    guesses: Vec<String>,
    answers: Vec<String>,
}

impl WordsList {
    pub fn load(/*cfg: WordleConfig*/) -> Self {
        let guesses = fs::read_to_string("./wordle/guesses.txt")
            .unwrap_or_else(|_| {
                fs::read_to_string("/wordle/guesses.txt")
                    .expect("guesses should be at ./wordle/guesses.txt or /wordle/guesses.txt")
            })
            .lines()
            .map(|s| s.to_owned())
            .collect::<Vec<String>>();

        assert!(!guesses.is_empty(), "guesses file should not be empty");

        let answers = fs::read_to_string("./wordle/answers.txt")
            .unwrap_or_else(|_| {
                fs::read_to_string("/wordle/answers.txt")
                    .expect("answers should be at ./wordle/answers.txt or /wordle/answers.txt")
            })
            .lines()
            .map(|s| s.to_owned())
            .collect::<Vec<String>>();

        Self { guesses, answers }
    }

    pub fn random_answer(&self) -> Word {
        let word = self
            .answers
            .choose(&mut rand::thread_rng())
            .expect("file should not be empty");

        Word::from_str(word).expect("file should contain only valid (5-letter) words")
    }

    pub fn valid_guess(&self, guess: &str) -> bool {
        self.guesses.contains(&guess.to_owned()) || self.answers.contains(&guess.to_owned())
    }
}

#[derive(Debug, Clone)]
pub struct DailyWordles {
    collection: Collection<DailyWordle>,
}

impl DailyWordles {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection("daily_wordles"),
        }
    }

    pub async fn latest(&self) -> DbResult<Option<DailyWordle>> {
        Ok(self
            .collection
            .find_one(
                None,
                FindOneOptions::builder()
                    .sort(doc! { "puzzle.number": -1 })
                    .build(),
            )
            .await?
            .filter(|puzzle| !puzzle.is_expired()))
    }

    pub async fn new_daily(&self, word: &Word) -> DbResult<DailyWordle> {
        let latest_number = self.latest().await?.map_or(0, |daily| daily.puzzle.number);

        let puzzle = puzzle::DailyPuzzle::new(latest_number + 1, word.clone());
        let wordle = DailyWordle::new(puzzle);

        self.collection.insert_one(&wordle, None).await?;

        Ok(wordle)
    }

    pub async fn update(&self, puzzle: u32, game: GameState) -> DbResult<()> {
        let user = mongodb::bson::ser::to_bson(&game.user).expect("implements serialize");
        let game = mongodb::bson::ser::to_bson(&game).expect("implements serialize");

        if self
            .collection
            .find_one(
                doc! {
                    "puzzle.number": puzzle,
                    "games": { "$elemMatch": { "user": &user } }
                },
                None,
            )
            .await?
            .is_some()
        {
            trace!("game exists in db");

            self.collection
                .update_one(
                    doc! {
                        "puzzle.number": puzzle,
                        "games": { "$elemMatch": { "user": &user } }
                    },
                    doc! { "$set": { "games.$": game } },
                    None,
                )
                .await?;
        } else {
            trace!("game does not exist in db");

            self.collection
                .update_one(
                    doc! { "puzzle.number": puzzle },
                    doc! { "$addToSet": {
                        "games": game
                    } },
                    None,
                )
                .await?;
        }

        Ok(())
    }

    async fn not_expired(&self) -> DbResult<Vec<DailyWordle>> {
        let mut vec = Vec::with_capacity(2);

        let mut cursor = self
            .collection
            .find(
                None,
                FindOptions::builder()
                    .sort(doc! { "puzzle.number":1 })
                    .limit(2)
                    .build(),
            )
            .await?;

        while let Some(daily) = cursor.next().await {
            if daily.as_ref().is_ok_and(|daily| daily.is_expired().not()) {
                vec.push(daily?);
            }
        }

        Ok(vec)
    }

    async fn playable_for(&self, user: UserId) -> DbResult<impl Iterator<Item = DailyWordle>> {
        Ok(self
            .not_expired()
            .await?
            .into_iter()
            .filter(move |daily| daily.is_playable_for(user)))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyWordle {
    puzzle: puzzle::DailyPuzzle,
    games: Vec<GameState>,
}

impl DailyWordle {
    fn new(puzzle: puzzle::DailyPuzzle) -> Self {
        Self {
            puzzle,
            games: Vec::new(),
        }
    }

    pub fn age_hours(&self) -> i64 {
        let age = Utc::now() - self.puzzle.started;
        age.num_hours()
    }

    pub fn is_recent(&self) -> bool {
        self.age_hours() < 24
    }

    pub fn is_old(&self) -> bool {
        self.age_hours() < 48 && !self.is_recent()
    }

    pub fn is_expired(&self) -> bool {
        self.age_hours() >= 48
    }

    pub fn user_game(&self, user: UserId) -> Option<&GameState> {
        self.games.iter().find(|game| game.user == user)
    }

    pub fn played_by(&self, user: UserId) -> bool {
        self.user_game(user).is_some()
    }

    pub fn finished_by(&self, user: UserId) -> bool {
        self.user_game(user).is_some_and(|game| game.is_finished())
    }

    pub fn is_playable_for(&self, user: UserId) -> bool {
        self.is_expired().not() && self.finished_by(user).not()
    }

    pub fn in_progress_for(&self, user: UserId) -> bool {
        self.user_game(user).is_some_and(|game| game.in_progress())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    user: UserId,
    guesses: Vec<Guess>,
    num_guesses: usize,
    finished: bool,
    solved: bool,
}

impl GameState {
    fn new(owner: UserId, guesses: &Vec<Guess>, finished: bool) -> Self {
        Self {
            user: owner,
            guesses: guesses.clone(),
            num_guesses: guesses.len(),
            finished,
            solved: guesses.last().map_or(false, |guess| guess.is_correct()),
        }
    }

    fn is_solved(&self) -> bool {
        self.guesses
            .last()
            .map_or(false, |guess| guess.is_correct())
    }

    fn unfinished(owner: UserId, guesses: &Vec<Guess>) -> Self {
        Self::new(owner, guesses, false)
    }

    fn finished(owner: UserId, guesses: &Vec<Guess>) -> Self {
        Self::new(owner, guesses, true)
    }

    fn into_finished(mut self) -> Self {
        self.finished = true;
        self
    }
    fn is_finished(&self) -> bool {
        self.finished
    }

    fn in_progress(&self) -> bool {
        self.is_finished().not()
    }
}

impl AsEmoji for GameState {
    fn as_emoji(&self) -> Cow<str> {
        self.guesses.as_emoji()
    }

    fn emoji_with_letters(&self) -> String {
        self.guesses.emoji_with_letters()
    }

    fn emoji_with_letters_spaced(&self) -> String {
        self.guesses.emoji_with_letters_spaced()
    }
}

use tokio::sync::{mpsc, oneshot};

use self::puzzle::DailyPuzzle;

fn create_menu(daily_available: bool) -> CreateReply {
    let menu_text = if daily_available {
        "you have a daily wordle available!"
    } else {
        "you do not have a daily wordle available! play a random wordle?"
    };

    CreateReply::new()
        .content(menu_text)
        .button(
            CreateButton::new("daily")
                .label("daily")
                .emoji(ReactionType::Unicode("📅".to_owned()))
                .style(poise::serenity_prelude::ButtonStyle::Primary)
                .disabled(!daily_available),
        )
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
        )
        .reply(true)
}

pub async fn play(
    ctx: crate::discord::commands::Context<'_>,
    mode: Option<GameType>,
    words: WordsList,
    dailies: DailyWordles,
    style: Option<GameStyle>,
    fix_flags: bool,
) -> Result<()> {
    let active_games = ctx.data().wordle().active_games();
    let read = active_games.read().await;
    if let Some((_, msg)) = read
        .iter()
        .find(|(channel, _)| *channel == ctx.channel_id())
    {
        ctx.reply(format!(
            "there's already a wordle game being played in this channel! [jump?]({})",
            msg.link()
        ))
        .await?;

        return Ok(());
    }

    drop(read);

    let owner = ctx.author();
    let in_guild = ctx.guild_id().is_some();

    // refresh daily puzzle
    let new_daily_word = words.random_answer();
    if let Some(daily) = dailies.latest().await? {
        if daily.is_old() {
            dailies.new_daily(&new_daily_word).await?;
        }
    } else {
        dailies.new_daily(&new_daily_word).await?;
    }

    let mut playable = dailies.playable_for(owner.id).await?.peekable();

    // only peeking at the value for now because the user might not consume it
    let next_daily = playable.peek();

    let (mode, mut menu, channel) = if let Some(mode) = mode {
        if next_daily.is_some() {
            if mode == GameType::Daily && in_guild {
                ctx.send(
                    CreateReply::new()
                        .content("you can't play a daily wordle in a server - check your dms!"),
                )
                .await?;
                let dm = owner.create_dm_channel(ctx).await?;

                (mode, dm.say(ctx, "loading...").await?, dm.id)
            } else {
                (
                    mode,
                    ctx.reply("loading...").await?.into_message().await?,
                    ctx.channel_id(),
                )
            }
        } else {
            ctx.reply(format!(
                "you don't have a daily puzzle available! check back in {} hours",
                24 - dailies
                    .latest()
                    .await?
                    .expect("at least one puzzle exists by now")
                    .age_hours()
            ))
            .await?;

            return Ok(());
        }
    } else {
        let menu_builder = create_menu(next_daily.is_some());
        let menu = ctx.send(menu_builder).await?.into_message().await?;

        if let Some(interaction) = menu.await_component_interaction(ctx).await {
            let channel = if interaction.data.custom_id.as_str() == "daily" && in_guild {
                owner.create_dm_channel(ctx).await?.id
            } else {
                ctx.channel_id()
            };

            let (mode, menu) = match interaction.data.custom_id.as_str() {
                "daily" => {
                    let message = if in_guild {
                        interaction
                            .create_response(ctx, CreateInteractionResponse::Acknowledge)
                            .await?;

                        ctx.send(
                            CreateReply::new()
                                .content(
                                    "you can't play a daily wordle in a server - check your dms!",
                                )
                                .ephemeral(true),
                        )
                        .await?;

                        menu.delete(ctx).await?;

                        channel.say(ctx, "loading daily wordle...").await?
                    } else {
                        interaction
                            .create_response(
                                ctx,
                                CreateInteractionResponse::UpdateMessage(
                                    CreateInteractionResponseMessage::new()
                                        .content("loading daily wordle...")
                                        .components(Vec::new()),
                                ),
                            )
                            .await?;

                        menu
                    };

                    (GameType::Daily, message)
                }
                "random" => {
                    interaction
                        .create_response(
                            ctx,
                            CreateInteractionResponse::UpdateMessage(
                                CreateInteractionResponseMessage::new()
                                    .content("loading random wordle...")
                                    .components(Vec::new()),
                            ),
                        )
                        .await?;

                    (GameType::Random, menu)
                }
                "cancel" => {
                    interaction
                        .create_response(
                            ctx,
                            CreateInteractionResponse::UpdateMessage(
                                CreateInteractionResponseMessage::new()
                                    .content("canceled!")
                                    .components(Vec::new()),
                            ),
                        )
                        .await?;

                    return Ok(());
                }
                _ => unreachable!(),
            };

            (mode, menu, channel)
        } else {
            panic!()
        }
    };

    let style = GameStyle::parse(style, fix_flags);

    let daily = match mode {
        GameType::Daily => playable.next(), // now we're consuming the playable puzzle, because the user wants it
        _ => None,
    };

    let puzzle = match mode {
        GameType::Random => Puzzle::random(&words),
        GameType::Daily => daily
            .clone()
            .expect("daily puzzle should be available")
            .puzzle
            .into(),
    };

    let title = match puzzle {
        Puzzle::Random(_) => "random wordle".to_owned(),
        Puzzle::Daily(DailyPuzzle { number, .. }) => format!("wordle {number}"),
    };

    let pause_cancel_button = match mode {
        GameType::Daily => CreateButton::new("pause")
            .emoji(ReactionType::Unicode("⏸️".to_owned()))
            .label("pause")
            .style(poise::serenity_prelude::ButtonStyle::Primary),
        GameType::Random => CreateButton::new("cancel")
            .emoji(ReactionType::Unicode("🚫".to_owned()))
            .label("cancel")
            .style(poise::serenity_prelude::ButtonStyle::Secondary),
    };

    let give_up_button = CreateButton::new("give_up")
        .emoji(ReactionType::Unicode("🏳️".to_owned()))
        .label("give up")
        .style(poise::serenity_prelude::ButtonStyle::Danger);

    let buttons = vec![pause_cancel_button, give_up_button];

    let action_row = CreateActionRow::Buttons(buttons.clone());

    let resumed = daily.and_then(|d| d.user_game(owner.id).cloned());

    let mut guesses = if let Some(ref resumed) = resumed {
        resumed.guesses.clone()
    } else {
        Vec::with_capacity(6)
    };

    let starting_emojis = resumed.map_or("no guesses yet!".to_owned(), |r| {
        r.emoji_with_style(style).into()
    });

    let game_msg = EditMessage::new()
        .content(format!("{title} {}/6\n{starting_emojis}", guesses.len()))
        .components(vec![action_row]);

    menu.edit(ctx, game_msg).await?;
    let mut game_msg = menu;

    let mut write = active_games.write().await;
    write.push((channel, game_msg.clone()));
    drop(write);

    let mut messages = channel.await_replies(ctx).stream();
    let mut interactions = game_msg.await_component_interactions(ctx).stream();

    let game_won = loop {
        tokio::select! {
            Some(msg) = messages.next() => {
                if let Some(guess) = handle_message(ctx, &msg, &words, &puzzle).await? {
                    guesses.push(guess.clone());
                    let state = GameState::unfinished(owner.id, &guesses);
                    let emojis = state.emoji_with_style(style);

                    game_msg.edit(ctx, EditMessage::new().content(format!("{title} {}/6\n{emojis}", guesses.len()))).await?;

                    if let Some(num) = puzzle.number() {
                        if state.is_solved() {
                            dailies.update(num, state.into_finished()).await?;
                            msg.reply(ctx, "you win!").await?;
                            break true;
                        } else {
                            dailies.update(num, state).await?;
                        }
                    } else if state.is_solved() {
                        msg.reply(ctx, "you win!").await?;
                        break true;
                    }
                }
            },
            Some(interaction) = interactions.next() => {
                if let Some(cmd) = handle_interaction(ctx, interaction, owner.id, &puzzle).await? {
                    match cmd {
                        WordleCommand::Pause => {
                            let state = GameState::unfinished(owner.id, &guesses);
                            dailies.update(puzzle.number()
                                .expect("this option is only available for daily puzzles"), state)
                                .await?;

                            break false;
                        }
                        WordleCommand::Cancel => { /* nothing to save */ break false; }
                        WordleCommand::GiveUp => {
                            if let Some(num) = puzzle.number() {
                                let state = GameState::finished(owner.id, &guesses);
                                dailies.update(num, state).await?;
                            }

                            break false;
                        }
                    }
                }
            }
        }
    };

    let disabled_buttons = buttons
        .iter()
        .cloned()
        .map(|button| button.disabled(true))
        .collect::<Vec<_>>();

    let final_content = &game_msg.content;
    let end_text = match game_won {
        true => "you win!",
        false => "game over!",
    };

    game_msg
        .edit(
            ctx,
            EditMessage::new()
                .components(vec![CreateActionRow::Buttons(disabled_buttons)])
                .content(format!("{final_content}\n{end_text}")),
        )
        .await?;

    if playable.next().is_some() {
        let notif_text = format!(
            "you have a new daily puzzle available! play it with `{}wordle daily`",
            ctx.prefix()
        );

        let mut components = Vec::with_capacity(1);

        // if the message is called in an application context,
        // this notification will be ephemeral, and can be dismissed.
        // however, if it's called with a prefix,
        // a way to delete the message must be added
        if matches!(ctx, Context::Prefix(_)) {
            let delete_button = CreateButton::new("delete")
                .emoji(ReactionType::Unicode("🗑️".to_owned()))
                .style(ButtonStyle::Secondary);

            components.push(CreateActionRow::Buttons(vec![delete_button]));
        }

        let notif = if channel == ctx.channel_id() {
            let notif_builder = CreateReply::default()
                .ephemeral(true)
                .content(notif_text)
                .components(components)
                .reply(true);

            ctx.send(notif_builder).await?.into_message().await?
        } else {
            let notif_builder = CreateMessage::default()
                .content(notif_text)
                .components(components);

            channel.send_message(ctx, notif_builder).await?
        };

        // handles the delete button if needed
        // times out after iter15m to avoid leaking memory
        if matches!(ctx, Context::Prefix(_)) {
            let notif_fut = async {
                if let Some(interaction) = notif.await_component_interaction(ctx).await {
                    if interaction.data.custom_id.as_str() == "delete" {
                        notif.delete(ctx).await?;
                    }
                }

                Ok::<(), crate::errors::Error>(())
            };

            if let Ok(fut) =
                tokio::time::timeout(tokio::time::Duration::from_secs(5 * 60), notif_fut).await
            {
                fut?;
            } else {
                notif.delete(ctx).await?;
            }
        }
    }

    let mut write = active_games.write().await;
    for (i, game) in write.clone().into_iter().enumerate() {
        if game.0 == channel {
            write.remove(i);
        }
    }

    Ok(())
}

async fn handle_message(
    cache_http: impl CacheHttp,
    msg: &Message,
    words: &WordsList,
    puzzle: &Puzzle,
) -> Result<Option<Guess>> {
    let content = msg.content.as_str();

    let QUESTION_MARK_REACT: ReactionType = ReactionType::Unicode("❓".to_owned());
    let CHECK_MARK_REACT: ReactionType = ReactionType::Unicode("✅".to_owned());
    let X_REACT: ReactionType = ReactionType::Unicode("❌".to_owned());

    // no need to check anything that doesn't look like a word
    if content.contains(" ").not() && content.chars().count() == 5 {
        if words.valid_guess(content) {
            msg.react(cache_http, CHECK_MARK_REACT).await?;
            return Ok(Some(puzzle.guess(content)));
        } else {
            msg.react(cache_http, QUESTION_MARK_REACT).await?;
        }
    }

    Ok(None)
}

async fn handle_interaction(
    cache_http: impl CacheHttp + AsRef<Http> + AsRef<ShardMessenger>,
    interaction: ComponentInteraction,
    owner: impl AsRef<UserId>,
    puzzle: &Puzzle,
) -> Result<Option<WordleCommand>> {
    let QUESTION_MARK_REACT: ReactionType = ReactionType::Unicode("❓".to_owned());
    let CHECK_MARK_REACT: ReactionType = ReactionType::Unicode("✅".to_owned());
    let X_REACT: ReactionType = ReactionType::Unicode("❌".to_owned());

    let blank_confirm_message = CreateInteractionResponseMessage::new()
        .button(
            CreateButton::new("yes")
                .emoji(CHECK_MARK_REACT)
                .label("yes")
                .style(poise::serenity_prelude::ButtonStyle::Secondary),
        )
        .button(
            CreateButton::new("no")
                .emoji(X_REACT)
                .label("no")
                .style(poise::serenity_prelude::ButtonStyle::Secondary),
        )
        .ephemeral(true);

    Ok(if interaction.user.id == *owner.as_ref() {
        match interaction.data.custom_id.as_str() {
            "cancel" => {
                let confirm_message = blank_confirm_message.content("really cancel?");

                interaction
                    .create_response(
                        &cache_http,
                        CreateInteractionResponse::Message(confirm_message),
                    )
                    .await?;

                if interaction
                    .get_response(&cache_http)
                    .await?
                    .await_component_interaction(&cache_http)
                    .await
                    .is_some()
                {
                    interaction.delete_response(&cache_http).await?;

                    interaction
                        .create_followup(
                            &cache_http,
                            CreateInteractionResponseFollowup::new().content("canceled!"),
                        )
                        .await?;

                    Some(WordleCommand::Cancel)
                } else {
                    None
                }
            }
            "pause" => {
                interaction
                    .create_response(
                        &cache_http,
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new()
                                .content("your game has been saved!"),
                        ),
                    )
                    .await?;

                Some(WordleCommand::Pause)
            }
            "give_up" => {
                let confirm_message = blank_confirm_message.content("really give up?");

                interaction
                    .create_response(
                        &cache_http,
                        CreateInteractionResponse::Message(confirm_message),
                    )
                    .await?;

                if interaction
                    .get_response(&cache_http)
                    .await?
                    .await_component_interaction(&cache_http)
                    .await
                    .is_some()
                {
                    let give_up_text = format!("the word was: {}", puzzle.answer());

                    interaction.delete_response(&cache_http).await?;

                    interaction
                        .create_followup(
                            &cache_http,
                            CreateInteractionResponseFollowup::new().content(give_up_text),
                        )
                        .await?;

                    Some(WordleCommand::GiveUp)
                } else {
                    None
                }
            }
            _ => None,
        }
    } else {
        match interaction.data.custom_id.as_str() {
            "cancel" => {
                interaction
                    .create_response(
                        &cache_http,
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new()
                                .ephemeral(true)
                                .content("you can only cancel games you started!"),
                        ),
                    )
                    .await?
            }
            "pause" => {
                interaction
                    .create_response(
                        &cache_http,
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new()
                                .ephemeral(true)
                                .content("you can only pause games you started!"),
                        ),
                    )
                    .await?
            }
            "give_up" => {
                interaction
                    .create_response(
                        &cache_http,
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new()
                                .ephemeral(true)
                                .content("you can only give up on games you started!"),
                        ),
                    )
                    .await?
            }
            _ => (),
        }

        None
    })
}

enum WordleCommand {
    Cancel,
    Pause,
    GiveUp,
}

#[derive(poise::ChoiceParameter, Debug, Clone, Copy, PartialEq)]
pub enum GameType {
    #[name = "daily"]
    Daily,
    #[name = "random"]
    Random,
}

#[derive(poise::ChoiceParameter, Debug, Clone, Copy, Default)]
pub enum GameStyle {
    #[name = "colors only"]
    #[name = "colors"]
    #[name = "colors_only"]
    #[name = "hidden"]
    Colors,
    #[name = "with letters"]
    #[name = "letters"]
    #[name = "with_letters"]
    #[name = "anx"]
    #[default]
    Letters,
    #[name = "spaced letters"]
    #[name = "spaced_letters"]
    #[name = "spaced"]
    #[name = "with spaces"]
    #[name = "with_spaces"]
    #[name = "letters with spaces"]
    #[name = "letters_with_spaces"]
    #[name = "fix flags"]
    #[name = "fix_flags"]
    SpacedLetters,
}

impl GameStyle {
    fn parse(style: Option<Self>, fix_flags: bool) -> Self {
        if fix_flags {
            Self::SpacedLetters
        } else {
            style.unwrap_or_default()
        }
    }
}

trait CreateReplyExt: Default {
    fn new() -> Self {
        Self::default()
    }

    fn button(self, button: CreateButton) -> Self;
}

impl CreateReplyExt for CreateReply {
    fn button(mut self, button: CreateButton) -> Self {
        if let Some(ref mut rows) = self.components {
            if let Some(buttons) = rows.iter_mut().find_map(|row| match row {
                CreateActionRow::Buttons(b) => Some(b),
                _ => None,
            }) {
                buttons.push(button);
            } else {
                rows.push(CreateActionRow::Buttons(vec![button]));
            }
        } else {
            self = self.components(vec![CreateActionRow::Buttons(vec![button])]);
        }

        self
    }
}
