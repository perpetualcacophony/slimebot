use super::{
    core::{Guess, Word},
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

    pub fn daily(number: u32, answer: Word) -> Self {
        Self::Daily(DailyPuzzle::new(number, answer))
    }

    pub fn as_daily(&self) -> Option<&DailyPuzzle> {
        match self {
            Self::Daily(daily) => Some(daily),
            _ => None,
        }
    }

    pub fn guess(&self, word: &str) -> Guess {
        self.answer().guess(word)
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
