use std::{
    borrow::Cow,
    collections::HashMap,
    fs,
    ops::{Index, IndexMut},
    path::Display,
    slice::Iter,
    str::FromStr,
};

use chrono::Utc;
use mongodb::{
    bson::{bson, doc, Bson},
    Collection, Database,
};
use poise::serenity_prelude::{model::user, UserId};
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
struct Word {
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
                    *count -= 1
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

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Game {
    user: UserId,
    pub answer: String,
    guesses: Vec<Guess>,
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
            answer: word,
            guesses: Vec::with_capacity(6),
        }
    }

    pub fn guess(&mut self, word: &str) {
        let answer = Word::new(&self.answer);
        let guess = answer.guess(word);
        self.guesses.push(guess);
    }

    pub fn guesses(&self) -> usize {
        self.guesses.len()
    }

    pub fn last_guess(&self) -> &Guess {
        self.guesses
            .last()
            .expect("should have guessed at least once")
    }

    pub fn won(&self) -> bool {
        self.last_guess().all_correct()
    }

    pub fn emoji(&self) -> String {
        self.guesses
            .iter()
            .fold(String::new(), |acc, guess| acc + "\n" + &guess.emoji())
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DailyPuzzle {
    #[serde(rename = "_id")]
    pub number: u32,
    pub started: UtcDateTime,
    answer: String,
    completed: Vec<Game>,
}

impl DailyPuzzle {
    fn new(words: &WordsList, number: u32) -> Self {
        let word = words.get_random().to_string();

        Self {
            number,
            started: Utc::now(),
            answer: word,
            ..Default::default()
        }
    }

    pub fn play(&self, user: UserId) -> Game {
        Game::from_word(user, &self.answer)
    }

    fn completed(&mut self, completion: Game) {
        assert!(completion.won(), "game should be completed");
        assert!(
            completion.answer == self.answer,
            "completion should have the same answer"
        );

        self.completed.push(completion);
    }

    pub fn completed_by(&self, user: UserId) -> bool {
        self.completed.iter().any(|game| game.user == user)
    }

    #[instrument(skip_all)]
    pub fn is_old(&self) -> bool {
        let time_since = Utc::now() - self.started;
        debug!(hours = time_since.num_hours());
        time_since.num_hours() >= PUZZLE_ACTIVE_HOURS && !self.is_expired()
    }

    pub fn is_expired(&self) -> bool {
        let time_since = Utc::now() - self.started;
        time_since.num_hours() >= 2 * PUZZLE_ACTIVE_HOURS
    }

    pub fn is_playable(&self, user: UserId) -> bool {
        !self.is_expired() && !self.completed_by(user)
    }

    pub fn is_backlogged(&self, user: UserId) -> bool {
        self.is_playable(user) && self.is_old()
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
pub struct DailyPuzzles(Collection<DailyPuzzle>);

impl DailyPuzzles {
    pub fn get(db: &Database) -> Self {
        let inner = db.collection("wordle_daily_puzzles");
        Self(inner)
    }

    pub fn collection(&self) -> &Collection<DailyPuzzle> {
        &self.0
    }

    pub async fn latest(&self) -> DbResult<Option<DailyPuzzle>> {
        self.collection().find_one(None, None).await
    }

    pub async fn new_puzzle(&self, words: &WordsList) -> DbResult<DailyPuzzle> {
        let latest = self.latest().await?;

        let number = if let Some(latest) = latest {
            latest.number + 1
        } else {
            1
        };

        let puzzle = DailyPuzzle::new(words, number);

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

            let time_since = Utc::now() - previous.started;
            if time_since.num_hours() < 2 * PUZZLE_ACTIVE_HOURS {
                Ok(previous)
            } else {
                Err(Error::Expired(previous))
            }
        })
    }

    pub async fn completed(&self, number: u32, game: Game) -> DbResult<()> {
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
}

trait AsEmoji {
    fn as_emoji(&self) -> Cow<str>;
}

impl<T> AsEmoji for Vec<T>
where
    T: AsEmoji,
{
    fn as_emoji(&self) -> Cow<str> {
        self.iter().map(|t| t.as_emoji()).collect::<String>().into()
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
    }
}
