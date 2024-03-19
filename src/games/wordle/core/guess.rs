use std::{
    borrow::Cow,
    convert::Infallible,
    ops::{Index, IndexMut},
    str::FromStr,
};

use serde::{Deserialize, Serialize};

use super::AsEmoji;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Guess {
    letters: Vec<(char, LetterState)>,
}

impl Guess {
    pub fn new(word: &str) -> Self {
        let letters = word
            .to_lowercase()
            .chars()
            .map(|ch: char| (ch, LetterState::NotPresent))
            .collect::<Vec<(char, LetterState)>>();

        Self { letters }
    }

    pub fn mark_correct(&mut self, index: usize) {
        self[index].1 = LetterState::Correct;
    }

    pub fn mark_wrong_place(&mut self, index: usize) {
        self[index].1 = LetterState::WrongPlace;
    }

    pub fn is_correct(&self) -> bool {
        self.letters
            .iter()
            .all(|(_, state)| *state == LetterState::Correct)
    }

    pub fn is_correct_at(&self, index: usize) -> bool {
        self[index].1 == LetterState::Correct
    }

    pub fn iter(&self) -> impl Iterator<Item = &(char, LetterState)> + '_ {
        self.letters.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut (char, LetterState)> + '_ {
        self.letters.iter_mut()
    }
}

impl AsEmoji for Guess {
    fn as_emoji(&self) -> Cow<str> {
        self.letters
            .iter()
            .map(|(_, letter)| *letter)
            .collect::<Vec<LetterState>>()
            .as_emoji()
            .into_owned()
            .into()
    }

    fn emoji_with_letters(&self) -> String {
        let (letters, states) = self.letters.iter().fold(
            (String::new(), String::new()),
            |(letters, states), (letter, state)| {
                (
                    letters + "‌" /* zero-width non-joiner */ + &letter.as_emoji(),
                    states + &state.as_emoji(),
                )
            },
        );

        letters + "\n" + &states
    }

    fn emoji_with_letters_spaced(&self) -> String {
        let (letters, states) = self.letters.iter().fold(
            (String::new(), String::new()),
            |(letters, states), (letter, state)| {
                (
                    letters + " " + &letter.as_emoji(),
                    states + " " + &state.as_emoji(),
                )
            },
        );

        letters.trim().to_owned() + "\n" + states.trim()
    }
}

impl IntoIterator for Guess {
    type Item = (char, LetterState);
    type IntoIter = std::vec::IntoIter<(char, LetterState)>;

    fn into_iter(self) -> Self::IntoIter {
        self.letters.into_iter()
    }
}

impl Index<usize> for Guess {
    type Output = (char, LetterState);

    fn index(&self, index: usize) -> &Self::Output {
        self.letters.index(index)
    }
}

impl IndexMut<usize> for Guess {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.letters.index_mut(index)
    }
}

impl ToString for Guess {
    fn to_string(&self) -> String {
        self.letters
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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum LetterState {
    #[default]
    NotPresent,
    WrongPlace,
    Correct,
}

impl AsEmoji for LetterState {
    fn as_emoji(&self) -> Cow<str> {
        match self {
            Self::Correct => "🟩",    // green square
            Self::WrongPlace => "🟨", // yellow square
            Self::NotPresent => "⬛", // black square
        }
        .into()
    }
}

impl FromStr for LetterState {
    type Err = Infallible;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(match s {
            "O" => Self::Correct,
            "o" => Self::WrongPlace,
            "." => Self::NotPresent,
            _ => Self::default(),
        })
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

#[cfg(test)]
mod tests {
    use paste::paste;

    macro_rules! string_match {
        ($($word:ident, $guess:ident => $result:expr;)+) => {
            use std::str::FromStr;

            paste! {
                $(
                    #[test]
                    fn [<$word _ $guess>]() {
                        let word = super::super::Word::from_str(&stringify!($word)).unwrap();
                        let guess = word.guess(&stringify!($guess));
                        pretty_assertions::assert_eq!(
                            guess, $result
                        )
                    }
                )+
            }
        };
    }

    string_match! {
        amber, amber => "OOOOO";
        amber, arbor => "O.O.O";
        amber, handy => ".o...";
        addra, opals => "..o..";
        mummy, tummy => ".OOOO";

        // these tests were made by annie!!
        vital, audio => "o..o.";
        scene, eager => "o..o.";
        today, level => ".....";
        phone, crown => "..O.o";
        royal, newly => "...oo";
        baker, dying => ".....";
        level, topic => ".....";
        blind, began => "O...o";
        movie, storm => "..o.o";
        spend, super => "O.oo.";
        still, worth => "...o.";
        build, usage => "o....";
        badly, alive => "oo...";
        harry, count => ".....";
        split, house => "...o.";
        quite, trust => "o.o..";
        flash, death => "..O.O";
        peter, crime => ".o..o";
        title, china => "..o..";
        these, smith => "o..oo";
        sport, lying => ".....";
        solve, shoot => "O.o..";
        prior, whole => "..o..";
        maybe, fruit => ".....";
        event, dealt => ".o..O";
    }
}
