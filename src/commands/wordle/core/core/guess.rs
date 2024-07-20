use poise::serenity_prelude::Message;
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet},
    convert::Infallible,
    fmt::Display,
    ops::{Deref, Index, IndexMut, Not},
    str::FromStr,
};
use tinyvec::TinyVec;

use super::super::words_list::WordsList;

use super::{AsEmoji, AsLetters};

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Guess {
    letters: [(char, LetterState); 5],
}

impl Guess {
    pub fn from_str(word: &impl AsLetters) -> Self {
        let letters = word
            .as_letters()
            .map(|ch: char| (ch.to_ascii_lowercase(), LetterState::NotPresent))
            .collect::<Vec<(char, LetterState)>>()
            .try_into()
            .expect("should have 5 letters");

        Self { letters }
    }

    pub fn new(partial: PartialGuess) -> Self {
        let letters = partial
            .letters
            .map(|ch| (ch.to_ascii_lowercase(), LetterState::NotPresent));

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

    pub fn as_slice(&self) -> &[(char, LetterState)] {
        self.as_ref()
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
    type IntoIter = std::array::IntoIter<(char, LetterState), 5>;

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

impl Display for Guess {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let txt = self
            .letters
            .iter()
            .map(|letter| letter.1.to_string())
            .collect::<String>();

        f.write_str(&txt)
    }
}

impl PartialEq<&str> for Guess {
    fn eq(&self, other: &&str) -> bool {
        &self.to_string() == other
    }
}

impl AsRef<[(char, LetterState)]> for Guess {
    fn as_ref(&self) -> &[(char, LetterState)] {
        &self.letters
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

impl Display for LetterState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Correct => "O",
            Self::WrongPlace => "o",
            Self::NotPresent => ".",
        })
    }
}

#[derive(Copy, Clone, Debug, Hash)]
pub struct PartialGuess {
    letters: [char; 5],
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum PartialGuessError {
    #[error("str has {0} chars, should have exactly 5")]
    WrongLength(usize),

    #[error("none of the valid words have symbols")]
    HasSymbols,

    #[error("'{0}' is not in the list of valid words")]
    NotInList(String),
}

pub trait ToPartialGuess {
    fn to_partial_guess(&self, words: &WordsList) -> Result<PartialGuess, PartialGuessError>;
}

impl ToPartialGuess for &str {
    fn to_partial_guess(&self, words: &WordsList) -> Result<PartialGuess, PartialGuessError> {
        let arr: [char; 5] = self
            .chars()
            .collect::<Vec<char>>()
            .try_into()
            .map_err(|_| PartialGuessError::WrongLength(self.chars().count()))?;

        for ch in arr {
            if ch.is_alphabetic().not() {
                return Err(PartialGuessError::HasSymbols);
            }
        }

        if words.get_word(self).is_none() {
            return Err(PartialGuessError::NotInList(self.to_string()));
        }

        Ok(PartialGuess { letters: arr })
    }
}

impl ToPartialGuess for String {
    fn to_partial_guess(&self, words: &WordsList) -> Result<PartialGuess, PartialGuessError> {
        self.as_str().to_partial_guess(words)
    }
}

impl ToPartialGuess for Message {
    fn to_partial_guess(&self, words: &WordsList) -> Result<PartialGuess, PartialGuessError> {
        self.content.to_partial_guess(words)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuessesRecord(Vec<kwordle::Guess>);

impl From<kwordle::Guesses> for GuessesRecord {
    fn from(value: kwordle::Guesses) -> Self {
        Self(value.into_vec())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct GuessesLimit(usize);

impl GuessesLimit {
    pub fn new(limit: usize) -> Self {
        assert!(limit != 0, "limit cannot be 0");
        Self(limit)
    }

    pub fn try_new(limit: usize) -> Option<Self> {
        (limit != 0).then_some(Self::new(limit))
    }
}

impl Display for GuessesLimit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.0))
    }
}

#[allow(clippy::from_over_into)]
impl Into<usize> for GuessesLimit {
    fn into(self) -> usize {
        self.0
    }
}

impl PartialEq<usize> for GuessesLimit {
    fn eq(&self, other: &usize) -> bool {
        &self.0 == other
    }
}

impl Default for GuessesLimit {
    fn default() -> Self {
        Self(6)
    }
}

#[derive(Clone, Debug, Default)]
pub struct Guesses {
    vec: TinyVec<[Guess; 6]>,
    pub limit: Option<GuessesLimit>,
}

impl Guesses {
    pub fn limit_reached(&self) -> bool {
        self.limit.is_some_and(|lim| lim == self.vec.len())
    }

    pub fn push(&mut self, guess: Guess) {
        if !self.limit_reached() {
            self.vec.push(guess)
        }
    }

    pub fn default_limit() -> Self {
        Self::new(GuessesLimit::default())
    }

    pub fn new(limit: impl Into<Option<GuessesLimit>>) -> Self {
        Self {
            limit: limit.into(),
            ..Self::default()
        }
    }

    pub fn unlimited() -> Self {
        Self::new(None)
    }
}

impl AsRef<[Guess]> for Guesses {
    fn as_ref(&self) -> &[Guess] {
        &self.vec
    }
}

pub trait GuessSlice: AsRef<[Guess]> {
    fn iter(&self) -> std::slice::Iter<Guess> {
        self.as_slice().iter()
    }

    fn as_slice(&self) -> &[Guess] {
        self.as_ref()
    }

    fn count(&self) -> usize {
        self.as_slice().len()
    }

    fn last(&self) -> Option<Guess> {
        self.as_slice().last().copied()
    }

    fn last_is_solved(&self) -> bool {
        self.last().is_some_and(|guess| guess.is_correct())
    }

    fn to_record(&self) -> GuessesRecord {
        self.as_slice().into()
    }

    fn letter_states(&self) -> LetterStates {
        self.iter()
            .flat_map(|guess| guess.as_slice())
            .copied()
            .collect()
    }

    fn used_letters(&self) -> CharSet {
        self.iter()
            .flat_map(|guess| guess.iter().map(|(ch, _)| *ch))
            .collect()
    }

    fn unused_letters(&self) -> CharSet {
        let used_letters = self.used_letters();
        ('a'..='z')
            .filter(|ch| !used_letters.contains(ch))
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct LetterStates(BTreeMap<char, LetterState>);

impl<T: Into<BTreeMap<char, LetterState>>> From<T> for LetterStates {
    fn from(value: T) -> Self {
        Self(value.into())
    }
}

impl FromIterator<(char, LetterState)> for LetterStates {
    fn from_iter<T: IntoIterator<Item = (char, LetterState)>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl Deref for LetterStates {
    type Target = BTreeMap<char, LetterState>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsEmoji for LetterStates {
    fn as_emoji(&self) -> Cow<str> {
        self.iter()
            .fold([String::new(), String::new()], |acc, letter_state| {
                [
                    acc[0].clone() + &letter_state.0.as_emoji() + " ",
                    acc[1].clone() + &letter_state.1.as_emoji() + " ",
                ]
            })
            .join("\n")
            .trim_end()
            .to_owned()
            .into()
    }
}

#[derive(Debug, Clone)]
pub struct CharSet(BTreeSet<char>);

impl Deref for CharSet {
    type Target = BTreeSet<char>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromIterator<char> for CharSet {
    fn from_iter<T: IntoIterator<Item = char>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl AsEmoji for CharSet {
    fn as_emoji(&self) -> Cow<str> {
        self.iter()
            .map(|ch| ch.as_emoji().to_string())
            .collect::<Vec<String>>()
            .join(", ")
            .into()
    }
}

impl GuessSlice for Guesses {}
impl GuessSlice for GuessesRecord {}

/*impl<T: GuessSlice> AsEmoji for T {
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
}*/

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
                        let guess = word.guess_str(&stringify!($guess));
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
