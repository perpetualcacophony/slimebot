use std::{
    borrow::Cow,
    collections::HashMap,
    fs,
    ops::{Index, IndexMut},
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
use poise::serenity_prelude::{futures::StreamExt, UserId};
use serde::{Deserialize, Serialize};
use tracing::{debug, instrument, trace};

use crate::UtcDateTime;

const PUZZLE_ACTIVE_HOURS: i64 = 24;

mod error;
pub use error::Error;

use mongodb::error::Error as MongoDbError;

type DbResult<T> = std::result::Result<T, MongoDbError>;
type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum LetterState {
    #[default]
    NotPresent,
    WrongPlace,
    Correct,
}

impl LetterState {
    fn emoji(&self) -> &'static str {
        match self {
            Self::Correct => "🟩",    // green square
            Self::WrongPlace => "🟨", // yellow square
            Self::NotPresent => "⬛", // black square
        }
    }
}

impl FromStr for LetterState {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(match s {
            "O" => Self::Correct,
            "o" => Self::WrongPlace,
            "." => Self::NotPresent,
            _ => Self::default(),
        })
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Word {
    word: Vec<char>,
    letters: HashMap<char, usize>,
}

impl Word {
    fn new(word: &str) -> Self {
        let word = word.to_lowercase().chars().collect::<Vec<char>>();

        assert!(word.len() == 5);

        let mut letters: HashMap<char, usize> = HashMap::new();

        for letter in word.iter() {
            if letters.contains_key(letter) {
                if let Some(count) = letters.get_mut(letter) {
                    *count += 1
                };
            } else {
                letters.insert(*letter, 1);
            }
        }

        Self { word, letters }
    }

    fn iter(&self) -> Iter<'_, char> {
        self.word.iter()
    }

    fn guess(&self, word: &str) -> Guess {
        let mut guess: Guess = Guess::new(word);
        debug!(?guess);
        debug!(answer = self.to_string());
        debug!(answer = ?self.letters);

        let mut letters = self.letters.clone();

        for (index, letter) in guess.clone().iter().copied().enumerate() {
            if self[index] == letter {
                guess.correct_at(index);
                let count = letters.get_mut(&letter).expect("word has letter");
                *count = count.saturating_sub(1);
            }
        }

        debug!(word, r = ?letters.get_mut(&'r'));
        debug!(word, o = ?letters.get_mut(&'o'));

        for (index, letter) in guess.clone().iter().copied().enumerate() {
            if letters.get(&letter).is_some_and(|count| *count > 0) {
                trace!("{letter}: wrong place");

                guess.has_letter_at(index);
                *letters.get_mut(&letter).expect("word has letter") -= 1;
            }
        }

        guess
    }
}

impl std::fmt::Display for Word {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.word.iter().collect::<String>())
    }
}

impl IntoIterator for Word {
    type Item = char;
    type IntoIter = std::vec::IntoIter<char>;

    fn into_iter(self) -> Self::IntoIter {
        self.word.into_iter()
    }
}

impl Index<usize> for Word {
    type Output = char;

    fn index(&self, index: usize) -> &Self::Output {
        self.word.index(index)
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Guess {
    word: Vec<(char, LetterState)>,
}

impl Guess {
    fn new(word: &str) -> Self {
        let word = word
            .to_lowercase()
            .chars()
            .map(|ch: char| (ch, LetterState::default()))
            .collect::<Vec<(char, LetterState)>>();

        Self { word }
    }

    fn correct_at(&mut self, index: usize) {
        self[index].1 = LetterState::Correct;
    }

    fn has_letter_at(&mut self, index: usize) {
        self[index].1 = LetterState::WrongPlace;
    }

    fn all_correct(&self) -> bool {
        for letter in &self.word {
            if letter.1 != LetterState::Correct {
                return false;
            }
        }

        true
    }

    fn iter(&self) -> impl Iterator<Item = &char> + '_ {
        self.word.iter().map(|letter| &letter.0)
    }

    pub fn emoji(&self) -> String {
        self.word
            .iter()
            .fold(String::new(), |acc, (_, letter)| acc + letter.emoji())
    }
}

impl Index<usize> for Guess {
    type Output = (char, LetterState);

    fn index(&self, index: usize) -> &Self::Output {
        self.word.index(index)
    }
}

impl IndexMut<usize> for Guess {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.word.index_mut(index)
    }
}

impl ToString for LetterState {
    fn to_string(&self) -> String {
        match self {
            Self::Correct => "O",
            Self::WrongPlace => "o",
            Self::NotPresent => ".",
        }
        .to_owned()
    }
}

impl ToString for Guess {
    fn to_string(&self) -> String {
        self.word
            .iter()
            .map(|letter| letter.1.to_string())
            .collect()
    }
}

impl PartialEq<&str> for Guess {
    fn eq(&self, other: &&str) -> bool {
        &self.to_string() == other
    }
}

#[derive(Debug, Clone, Default)]
pub struct Game {
    user: UserId,
    guesses: Vec<Guess>,
    pub answer: Word,
    pub started: StartTime,
    pub number: Option<u32>,
    pub ended: bool,
}

//const WORDS: &str = include_str!("wordle.txt");

impl Game {
    pub fn random(user: UserId, words: &WordsList) -> Self {
        Self::from_word(user, words.get_random())
    }

    pub fn from_word(user: UserId, word: impl Into<String>) -> Self {
        let word = word.into();

        assert!(word.len() == 5);

        Self {
            user,
            guesses: Vec::with_capacity(6),
            answer: Word::new(&word),
            started: StartTime::none(),
            number: None,
            ended: false,
        }
    }

    pub fn guess(&mut self, word: &str) {
        let guess = self.answer.guess(word);
        self.guesses.push(guess);
    }

    pub fn guesses(&self) -> usize {
        self.guesses.len()
    }

    pub fn last_guess(&self) -> Option<&Guess> {
        self.guesses.last()
    }

    pub fn solved(&self) -> bool {
        self.last_guess().is_some_and(|g| g.all_correct())
    }

    pub fn emoji(&self) -> String {
        self.guesses
            .iter()
            .map(|guess| guess.as_emoji())
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn results(&self, ended: bool) -> GameResult {
        GameResult {
            puzzle: self
                .number
                .expect("currently only supporting saving daily puzzles"),
            user: self.user,
            guesses: self.guesses.clone(),
            num_guesses: self.guesses(),
            solved: self.solved(),
            ended,
        }
    }

    pub fn is_daily(&self) -> bool {
        self.number.is_some()
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct GameResult {
    pub puzzle: u32,
    user: UserId,
    guesses: Vec<Guess>,
    num_guesses: usize,
    solved: bool,
    ended: bool,
}

impl AsEmoji for GameResult {
    fn as_emoji(&self) -> Cow<str> {
        self.guesses.as_emoji()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DailyPuzzle {
    #[serde(rename = "_id")]
    pub number: u32,
    pub started: StartTime,
    answer: String,
    finished: Vec<GameResult>,
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
    pub fn load() -> Self {
        let file = fs::read_to_string("./wordle.txt").unwrap_or_else(|_| {
            fs::read_to_string("/wordle.txt")
                .expect("words should be at ./wordle.txt or /wordle.txt")
        });

        let words = file.lines().map(|s| s.to_owned()).collect::<Vec<String>>();

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

impl From<&str> for Word {
    fn from(value: &str) -> Self {
        Word::new(value)
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
    pub words: WordsList,
}

impl DailyPuzzles {
    pub fn get(db: &Database, words: WordsList) -> Self {
        let collection = db.collection("wordle_daily_puzzles");
        Self { collection, words }
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

        let puzzle = DailyPuzzle::new(&self.words, number);

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

pub trait AsEmoji {
    fn as_emoji(&self) -> Cow<str>;
}

impl<T> AsEmoji for Vec<T>
where
    T: AsEmoji,
{
    fn as_emoji(&self) -> Cow<str> {
        self.iter()
            .map(|t| t.as_emoji())
            .collect::<Vec<_>>()
            .join("\n")
            .into()
    }
}

impl AsEmoji for Guess {
    fn as_emoji(&self) -> Cow<str> {
        self.emoji().into()
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

#[cfg(test)]
mod tests {
    use super::Word;
    use pretty_assertions::assert_eq;
    use tracing_test::traced_test;

    #[test]
    #[traced_test]
    fn win() {
        let word = Word::new("amber");
        assert!(word.guess("amber").all_correct())
    }

    #[test]
    #[traced_test]
    fn display() {
        let word = Word::new("amber");
        let guess = word.guess("amber");
        assert_eq!(guess, "OOOOO");

        let guess = word.guess("arbor");
        assert_eq!(guess, "O.O.O");

        let guess = word.guess("handy");
        assert_eq!(guess, ".o...");

        let word = Word::new("addra");
        let guess = word.guess("opals");
        assert_eq!(guess, "..o..")
    }
}
