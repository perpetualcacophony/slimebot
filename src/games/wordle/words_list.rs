use super::core::Word;
use rand::prelude::SliceRandom;
use std::{fs, str::FromStr};

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
