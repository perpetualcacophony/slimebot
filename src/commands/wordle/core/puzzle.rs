use std::borrow::Cow;

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::framework::data::UtcDateTime;

#[derive(Debug, Serialize, Clone)]
pub enum Puzzle {
    Random(#[serde(serialize_with = "kwordle::Word::serialize_as_str")] kwordle::Word<5>),
    Daily(DailyPuzzle),
}

impl Puzzle {
    pub fn random(words: &kwordle::WordsList<5>) -> Self {
        let answer = words.answers.random();

        Self::Random(answer)
    }

    #[allow(dead_code)] // this is used in a macro
    pub fn guess_str(&self, list: &kwordle::WordsList<5>, s: &str) -> kwordle::Guess<5> {
        self.answer().guess_str(list, s).unwrap()
    }

    pub fn guess(&self, word: &kwordle::Word<5>) -> kwordle::Guess<5> {
        self.answer().guess(*word)
    }

    pub fn is_daily(&self) -> bool {
        matches!(self, Self::Daily(..))
    }

    #[allow(dead_code)] // worth having for a complete API
    pub fn is_random(&self) -> bool {
        matches!(self, Self::Random(..))
    }

    pub fn answer(&self) -> &kwordle::Word<5> {
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

    pub fn from_partial(partial: PartialPuzzle, list: &kwordle::WordsList<5>) -> Option<Self> {
        match partial {
            PartialPuzzle::Random(string) => {
                Some(Puzzle::Random(kwordle::Word::from_str(list, &string).ok()?))
            }
            PartialPuzzle::Daily(partial) => {
                DailyPuzzle::from_partial(partial, list).map(Puzzle::Daily)
            }
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct DailyPuzzle {
    pub number: u32,

    #[serde(serialize_with = "kwordle::Word::serialize_as_str")]
    answer: kwordle::Word<5>,

    pub started: UtcDateTime,
}

impl DailyPuzzle {
    pub fn new(number: u32, answer: kwordle::Word<5>) -> Self {
        Self {
            number,
            answer,
            started: Utc::now(),
        }
    }

    pub fn from_partial(partial: PartialDailyPuzzle, list: &kwordle::WordsList) -> Option<Self> {
        Some(Self {
            number: partial.number,
            answer: kwordle::Word::from_str(list, &partial.answer).ok()?,
            started: partial.started,
        })
    }

    pub fn into_partial(self) -> PartialDailyPuzzle {
        PartialDailyPuzzle {
            number: self.number,
            answer: self.answer.to_string(),
            started: self.started,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialDailyPuzzle {
    pub number: u32,

    answer: String,

    pub started: UtcDateTime,
}

impl From<DailyPuzzle> for Puzzle {
    fn from(value: DailyPuzzle) -> Self {
        Self::Daily(value)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PartialPuzzle {
    Random(String),
    Daily(PartialDailyPuzzle),
}
