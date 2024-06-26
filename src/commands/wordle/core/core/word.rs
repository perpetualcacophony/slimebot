use std::{collections::HashMap, ops::Index, str::FromStr};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, trace};

use crate::commands::wordle::core::core;

use core::{
    guess::{Guess, LetterState, PartialGuess},
    AsLetters,
};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Word {
    pub letters: Vec<char>,
    letter_counts: HashMap<char, usize>,
}

impl Word {
    pub fn parse(s: &str) -> Word {
        let letters = s.to_lowercase().chars().collect::<Vec<char>>();

        assert!(letters.len() == 5, "word must have exactly 5 letters");

        let mut letter_counts: HashMap<char, usize> = HashMap::new();
        for letter in letters.iter() {
            if let Some(count) = letter_counts.get_mut(letter) {
                *count += 1;
            } else {
                letter_counts.insert(*letter, 1);
            }
        }

        Self {
            letters,
            letter_counts,
        }
    }

    pub fn guess(&self, partial: PartialGuess) -> Guess {
        let mut letter_counts = self.letter_counts.clone();

        let mut guess = Guess::new(partial);

        for (index, (letter, state)) in guess.iter_mut().enumerate() {
            if self[index] == *letter {
                *state = LetterState::Correct;
                let count = letter_counts.get_mut(letter).expect("word has letter");
                *count = count.saturating_sub(1);
            }
        }

        for (letter, state) in guess.iter_mut() {
            if *state != LetterState::Correct
                && letter_counts.get(letter).is_some_and(|count| *count > 0)
            {
                trace!("{}: wrong place", letter);

                *state = LetterState::WrongPlace;
                *letter_counts.get_mut(letter).expect("word has letter") -= 1;
            }
        }

        guess
    }

    pub fn guess_str(&self, word: &impl AsLetters) -> Guess {
        let mut guess: Guess = Guess::from_str(word);
        debug!(?guess);
        debug!(answer = self.to_string());
        debug!(answer = ?self.letter_counts);

        let mut letter_counts = self.letter_counts.clone();

        for (index, (letter, state)) in guess.iter_mut().enumerate() {
            if self[index] == *letter {
                *state = LetterState::Correct;
                let count = letter_counts.get_mut(letter).expect("word has letter");
                *count = count.saturating_sub(1);
            }
        }

        for (letter, state) in guess.iter_mut() {
            if *state != LetterState::Correct
                && letter_counts.get(letter).is_some_and(|count| *count > 0)
            {
                trace!("{}: wrong place", letter);

                *state = LetterState::WrongPlace;
                *letter_counts.get_mut(letter).expect("word has letter") -= 1;
            }
        }

        guess
    }

    pub fn as_str(&self) -> String {
        self.letters
            .iter()
            .map(|ch| ch.to_string())
            .collect::<Vec<_>>()
            .join("")
    }
}

#[derive(Error, Debug)]
#[error("word `{0}` must have 5 letters but has {}", .0.chars().count())]
pub struct ParseWordError(String);

impl FromStr for Word {
    type Err = ParseWordError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let letters = s.to_lowercase().chars().collect::<Vec<char>>();

        if s.chars().count() != 5 {
            return Err(ParseWordError(s.to_owned()));
        }

        let mut letter_counts: HashMap<char, usize> = HashMap::new();
        for letter in letters.iter() {
            if let Some(count) = letter_counts.get_mut(letter) {
                *count += 1;
            } else {
                letter_counts.insert(*letter, 1);
            }
        }

        Ok(Self {
            letters,
            letter_counts,
        })
    }
}

impl TryFrom<String> for Word {
    type Error = ParseWordError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(&value)
    }
}

impl From<Word> for String {
    fn from(value: Word) -> Self {
        value.to_string()
    }
}

impl std::fmt::Display for Word {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.letters.iter().collect::<String>())
    }
}

impl IntoIterator for Word {
    type Item = char;
    type IntoIter = std::vec::IntoIter<char>;

    fn into_iter(self) -> Self::IntoIter {
        self.letters.into_iter()
    }
}

impl Index<usize> for Word {
    type Output = char;

    fn index(&self, index: usize) -> &Self::Output {
        self.letters.index(index)
    }
}
