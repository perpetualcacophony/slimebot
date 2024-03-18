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
        CacheHttp, ChannelId, ComponentInteraction, CreateActionRow, CreateButton,
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

    pub fn is_completed_by(&self, user: UserId) -> bool {
        self.games.iter().any(|game| game.user == user)
    }

    pub fn is_playable_for(&self, user: UserId) -> bool {
        self.is_expired().not() && self.is_completed_by(user).not()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    user: UserId,
    guesses: Vec<Guess>,
}

impl GameState {
    fn new(owner: UserId, guesses: &Vec<Guess>) -> Self {
        Self {
            user: owner,
            guesses: guesses.clone(),
        }
    }
}

impl AsEmoji for GameState {
    fn as_emoji(&self) -> Cow<str> {
        self.guesses.as_emoji()
    }

    fn emoji_with_letter(&self) -> String {
        self.guesses.emoji_with_letter()
    }
}

use tokio::sync::{mpsc, oneshot};

use self::puzzle::DailyPuzzle;

struct WordlePool {
    rx: mpsc::Receiver<PoolMessage>,
    games: Vec<u64>,
}

impl WordlePool {
    fn create() -> (WordlePool, WordleHandle) {
        let (tx, rx) = mpsc::channel(32);

        let pool = WordlePool {
            rx,
            games: Vec::new(),
        };
        let handle = WordleHandle { tx };

        (pool, handle)
    }

    /*
    fn setup() -> (impl Future<Output = ()>, WordleHandle) {
        let (pool, handle) = Self::create();

        (Self::task(pool), handle)
    }

    async fn task(mut pool: WordlePool) {
        while let Some(msg) = pool.rx.recv().await {
            match msg {
                PoolMessage::NewGame {
                    tx,
                    puzzle,
                    channel,
                } => {
                    let result = if pool.game_in_channel(channel) {
                        Err(anyhow!("channel already has game").into())
                    } else {
                        let (game, handle) = Game::create(puzzle, channel);
                        pool.games.push(game);

                        Ok(handle)
                    };

                    tx.send(result);
                }
            };
        }
    }
    */

    fn add_game(&mut self, id: u64) {
        self.games.push(id)
    }

    fn game_in_channel(&self, id: ChannelId) -> bool {
        self.games.iter().any(|game| *game == id.get())
    }
}

struct WordleHandle {
    tx: mpsc::Sender<PoolMessage>,
}

enum PoolMessage {
    NewGame {
        tx: oneshot::Sender<Result<Game>>,
        puzzle: Puzzle,
        channel: ChannelId,
    },
}

struct Game {
    puzzle: Puzzle,
    channel: ChannelId,
    owner: UserId,
    guesses: Vec<Guess>,
}

async fn create_menu(
    ctx: crate::discord::commands::Context<'_>,
    channel: ChannelId,
    daily_available: bool,
) -> Result<Message> {
    let menu_text = if daily_available {
        "you have a daily wordle available!"
    } else {
        "you do not have a daily wordle available! play a random wordle?"
    };

    let builder = CreateMessage::new()
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
        );

    Ok(channel.send_message(ctx, builder).await?)
}

pub async fn play(
    ctx: crate::discord::commands::Context<'_>,
    daily: bool,
    words: WordsList,
    dailies: DailyWordles,
    anx: bool,
) -> Result<()> {
    let owner = ctx.author();

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

    let next_daily = playable.peek();

    // menu goes HERE
    let mut menu = create_menu(ctx, ctx.channel_id(), next_daily.is_some()).await?;

    let play_daily = if let Some(interaction) = menu.await_component_interaction(ctx).await {
        match interaction.data.custom_id.as_str() {
            "daily" => {
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

                true
            }
            "random" => {
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

                false
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

                return Err(crate::errors::Error::Manual(anyhow!("fghkjfdhkdgjfghd")));
            }
            _ => unreachable!(),
        }
    } else {
        panic!()
    };

    let puzzle = if !play_daily {
        Puzzle::random(&words)
    } else {
        playable
            .next()
            .expect("daily puzzle should be available")
            .puzzle
            .into()
    };

    let title = match puzzle {
        Puzzle::Random(_) => "random wordle".to_owned(),
        Puzzle::Daily(DailyPuzzle { number, .. }) => format!("wordle {number}"),
    };

    let mut buttons = Vec::with_capacity(2);

    buttons.push(if daily {
        CreateButton::new("pause")
            .emoji(ReactionType::Unicode("⏸️".to_owned()))
            .label("pause")
            .style(poise::serenity_prelude::ButtonStyle::Primary)
    } else {
        CreateButton::new("cancel")
            .emoji(ReactionType::Unicode("🚫".to_owned()))
            .label("cancel")
            .style(poise::serenity_prelude::ButtonStyle::Secondary)
    });

    buttons.push(
        CreateButton::new("give_up")
            .emoji(ReactionType::Unicode("🏳️".to_owned()))
            .label("give up")
            .style(poise::serenity_prelude::ButtonStyle::Danger),
    );

    let action_row = CreateActionRow::Buttons(buttons.clone());

    let game_msg = EditMessage::new()
        .content(format!("{title}\nno guesses yet!"))
        .components(vec![action_row]);

    menu.edit(ctx, game_msg).await?;
    let mut game_msg = menu;

    let mut messages = ctx.channel_id().await_replies(ctx).stream();
    let mut interactions = game_msg.await_component_interactions(ctx).stream();

    let mut guesses: Vec<Guess> = Vec::new();

    loop {
        tokio::select! {
            Some(msg) = messages.next() => {
                if let Some(guess) = handle_message(ctx, &msg, &words, &puzzle).await? {
                    guesses.push(guess.clone());
                    let state = GameState::new(owner.id, &guesses);
                    let emojis = if anx {
                        state.emoji_with_letter()
                    } else {
                        state.as_emoji().into_owned()
                    };

                    game_msg.edit(ctx, EditMessage::new().content(format!("{title} {}/6\n{emojis}", guesses.len()))).await?;

                    if let Some(num) = puzzle.number() {
                        dailies.update(num, state).await?;
                    }

                    if guess.is_correct() {
                        msg.reply(ctx, "you win!").await?;
                        break;
                    }
                }
            },
            Some(interaction) = interactions.next() => {
                if let Some(cmd) = handle_interaction(ctx, interaction, owner.id, &puzzle).await? {
                    break;
                }
            }
        }
    }

    let disabled_buttons = buttons
        .iter()
        .cloned()
        .map(|button| button.disabled(true))
        .collect::<Vec<_>>();

    game_msg
        .edit(
            ctx,
            EditMessage::new().components(vec![CreateActionRow::Buttons(disabled_buttons)]),
        )
        .await?;

    if playable.next().is_some() {
        ctx.reply("you have a new daily puzzle available! play it with `!!wordle daily`")
            .await?;
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
            "pause" => Some(WordleCommand::Pause),
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
