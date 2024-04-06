use std::borrow::Cow;

use super::{
    core::{AsLetters, Guess, PartialGuess, Word},
    WordsList,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::UtcDateTime;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Puzzle {
    Random(Word),
    Daily(DailyPuzzle),
}

impl Puzzle {
    pub fn random(words: &WordsList) -> Self {
        let answer = words.random_answer();

        Self::Random(answer)
    }

    pub fn guess_str(&self, word: &impl AsLetters) -> Guess {
        self.answer().guess_str(word)
    }

    pub fn guess(&self, partial: PartialGuess) -> Guess {
        self.answer().guess(partial)
    }

    pub fn is_daily(&self) -> bool {
        matches!(self, Self::Daily(..))
    }

    pub fn answer(&self) -> &Word {
        match self {
            Self::Random(answer) => answer,
            Self::Daily(daily) => &daily.answer,
        }
    }

    pub fn number(&self) -> Option<u32> {
        match self {
            Self::Random(_) => None,
            Self::Daily(daily) => Some(daily.number),
        }
    }

    pub fn title(&self) -> Cow<str> {
        match self {
            Self::Random(..) => "random wordle".into(),
            Self::Daily(DailyPuzzle { number, .. }) => format!("daily wordle {number}").into(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DailyPuzzle {
    pub number: u32,
    answer: Word,
    pub started: UtcDateTime,
}

impl DailyPuzzle {
    pub fn new(number: u32, answer: Word) -> Self {
        Self {
            number,
            answer,
            started: Utc::now(),
        }
    }
}

impl From<DailyPuzzle> for Puzzle {
    fn from(value: DailyPuzzle) -> Self {
        Self::Daily(value)
    }
}
