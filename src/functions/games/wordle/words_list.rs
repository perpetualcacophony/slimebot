use super::core::Word;
use rand::{prelude::SliceRandom, seq::IteratorRandom};
use std::{collections::HashSet, fs};

#[derive(Debug, Clone)]
pub struct WordsList {
    guesses: HashSet<String>,
    answers: HashSet<String>,
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
            .collect::<HashSet<String>>();

        assert!(!guesses.is_empty(), "guesses file should not be empty");

        let answers = fs::read_to_string("./wordle/answers.txt")
            .unwrap_or_else(|_| {
                fs::read_to_string("/wordle/answers.txt")
                    .expect("answers should be at ./wordle/answers.txt or /wordle/answers.txt")
            })
            .lines()
            .map(|s| s.to_owned())
            .collect::<HashSet<String>>();

        Self { guesses, answers }
    }

    pub fn random_answer(&self) -> Word {
        let word = self
            .answers
            .iter()
            .choose(&mut rand::thread_rng())
            .expect("file should not be empty");

        Word::parse(word)
    }

    pub fn valid_guess(&self, guess: &str) -> bool {
        let guess = &guess.to_lowercase();

        self.guesses.contains(guess) || self.answers.contains(guess)
    }

    pub fn get_word(&self, s: &str) -> Option<Word> {
        self.valid_guess(s).then_some(Word::parse(s))
    }
}

impl AsRef<WordsList> for WordsList {
    fn as_ref(&self) -> &WordsList {
        self
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::WordsList;

    #[test]
    fn fetch_answers() {
        let words = ["amber", "mummy", "opals", "sonar", "today"];

        let list = WordsList {
            guesses: HashSet::new(),
            answers: HashSet::from_iter(words.map(|s| s.to_owned())),
        };

        for _ in 0..10 {
            let answer = list.random_answer();
            assert!(words.contains(&answer.to_string().as_str()))
        }
    }

    #[test]
    fn fetch_guesses() {
        let words = ["amber", "mummy", "opals", "sonar", "today"];

        let list = WordsList {
            guesses: HashSet::new(),
            answers: HashSet::from_iter(words.map(|s| s.to_owned())),
        };

        for word in words {
            assert!(list.valid_guess(word))
        }
    }
}
