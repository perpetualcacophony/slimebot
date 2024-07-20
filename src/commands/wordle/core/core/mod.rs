use std::borrow::Cow;

use poise::serenity_prelude::UserId;
use serde::{Deserialize, Serialize};

mod word;
pub use word::Word;

pub mod guess;
pub use guess::{Guess, Guesses, GuessesRecord, PartialGuess, PartialGuessError, ToPartialGuess};

use self::guess::LetterState;

use super::game::options::GameStyle;

/*
#[derive(Debug, Clone, Default)]
pub struct Game {
    pub user: UserId,
    guesses: Vec<Guess>,
    pub answer: Word,
    pub started: StartTime,
    pub number: Option<u32>,
    pub ended: bool,
}

pub struct GameNew<'a> {
    puzzle: Option<&'a DailyPuzzle>,
}

impl Game {
    pub fn random(user: UserId, words: &WordsListNew) -> Self {
        Self::from_word(user, words.random_answer())
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
        self.last_guess().is_some_and(|g| g.is_correct())
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
*/

pub trait AsEmoji {
    fn as_emoji(&self) -> Cow<str>;

    fn emoji_with_letters(&self) -> String {
        self.as_emoji().into()
    }

    fn emoji_with_letters_spaced(&self) -> String {
        self.emoji_with_letters()
    }

    fn emoji_with_style(&self, style: GameStyle) -> Cow<str> {
        match style {
            GameStyle::Colors => self.as_emoji(),
            GameStyle::Letters => self.emoji_with_letters().into(),
            GameStyle::SpacedLetters => self.emoji_with_letters_spaced().into(),
        }
    }

    #[allow(dead_code)]
    fn emoji_default_style(&self) -> String {
        self.emoji_with_style(GameStyle::default()).into()
    }
}

impl AsEmoji for char {
    fn as_emoji(&self) -> Cow<str> {
        let alphabet_letters = 'a'..='z';
        let emoji_letters = '🇦'..='🇿';

        let emoji = alphabet_letters
            .zip(emoji_letters)
            .find_map(|(letter, emoji)| (*self == letter).then_some(emoji))
            .expect("char should be alphabetic");

        emoji.to_string().into()
    }
}

impl AsEmoji for Vec<LetterState> {
    fn as_emoji(&self) -> Cow<str> {
        self.iter()
            .map(|l| l.as_emoji())
            .collect::<Vec<_>>()
            .join("")
            .into()
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct GameResults {
    user: UserId,
    guesses: Vec<kwordle::Guess>,
    num_guesses: usize,
    solved: bool,
    ended: bool,
}

pub trait AsLetters {
    fn as_letters(&self) -> impl Iterator<Item = char>;
}

impl AsLetters for &str {
    fn as_letters(&self) -> impl Iterator<Item = char> {
        self.chars()
    }
}

impl AsLetters for Word {
    fn as_letters(&self) -> impl Iterator<Item = char> {
        self.letters.iter().copied()
    }
}

impl AsEmoji for kwordle::LetterState {
    fn as_emoji(&self) -> Cow<str> {
        match self {
            Self::Correct => "🟩",    // green square
            Self::WrongPlace => "🟨", // yellow square
            Self::NotPresent => "⬛", // black square
        }
        .into()
    }
}

impl AsEmoji for Vec<kwordle::LetterState> {
    fn as_emoji(&self) -> Cow<str> {
        self.iter()
            .map(|l| l.as_emoji())
            .collect::<Vec<_>>()
            .join("")
            .into()
    }
}

impl AsEmoji for kwordle::Guess {
    fn as_emoji(&self) -> Cow<str> {
        self.into_iter()
            .map(|letter| letter.state())
            .collect::<Vec<kwordle::LetterState>>()
            .as_emoji()
            .into_owned()
            .into()
    }

    fn emoji_with_letters(&self) -> String {
        let (letters, states) =
            self.into_iter()
                .fold((String::new(), String::new()), |(letters, states), l| {
                    (
                        letters + "‌" /* zero-width non-joiner */ + &l.letter().as_emoji(),
                        states + &l.state().as_emoji(),
                    )
                });

        letters + "\n" + &states
    }

    fn emoji_with_letters_spaced(&self) -> String {
        let (letters, states) =
            self.into_iter()
                .fold((String::new(), String::new()), |(letters, states), l| {
                    (
                        letters + " " + &l.letter().as_emoji(),
                        states + " " + &l.state().as_emoji(),
                    )
                });

        letters.trim().to_owned() + "\n" + states.trim()
    }
}

impl AsEmoji for kwordle::Guesses {
    fn as_emoji(&self) -> Cow<str> {
        self.iter()
            .map(|g| g.as_emoji())
            .collect::<Vec<_>>()
            .join("\n")
            .into()
    }

    fn emoji_with_letters(&self) -> String {
        self.iter()
            .map(|g| g.emoji_with_letters())
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn emoji_with_letters_spaced(&self) -> String {
        self.iter()
            .map(|g| g.emoji_with_letters_spaced())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl AsEmoji for kwordle::Letter {
    fn as_emoji(&self) -> Cow<str> {
        let alphabet_letters = kwordle::letter::ALPHABET;
        let emoji_letters = '🇦'..='🇿';

        let emoji = alphabet_letters
            .zip(emoji_letters)
            .find_map(|(letter, emoji)| (*self == letter).then_some(emoji))
            .expect("char should be alphabetic");

        emoji.to_string().into()
    }
}

impl AsEmoji for std::collections::BTreeSet<kwordle::Letter> {
    fn as_emoji(&self) -> Cow<str> {
        self.into_iter()
            .map(kwordle::Letter::as_emoji)
            .collect::<Vec<_>>()
            .join(", ")
            .into()
    }
}
