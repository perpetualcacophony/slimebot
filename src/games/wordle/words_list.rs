use super::core::Word;
use rand::prelude::SliceRandom;
use std::fs;

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

#[cfg(test)]
mod tests {
    use super::WordsList;

    #[test]
    fn fetch_answers() {
        let words = ["amber", "mummy", "opals", "sonar", "today"].map(|s| s.to_owned());

        let list = WordsList {
            guesses: Vec::new(),
            answers: words.to_vec(),
        };

        for _ in 0..10 {
            let answer = list.random_answer();
            assert!(words.contains(&answer.to_string()))
        }
    }

    #[test]
    fn fetch_guesses() {
        let words = ["amber", "mummy", "opals", "sonar", "today"];

        let list = WordsList {
            guesses: Vec::new(),
            answers: words.map(|s| s.to_owned()).to_vec(),
        };

        for word in words {
            assert!(list.valid_guess(word))
        }
    }
}
