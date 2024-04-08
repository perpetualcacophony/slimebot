use std::{num::NonZeroI8, ops::Neg};

use rand::{rngs::StdRng, seq::IteratorRandom, Rng, SeedableRng};
use regex::Regex;

use thiserror::Error;
use tracing::{debug, instrument, trace};

#[derive(Debug, Error, PartialEq)]
pub enum DiceRollError {
    #[error("")]
    NoFaces,
    #[error("")]
    InvalidExtra(String),
    #[error("")]
    InvalidExtraSign(String),
    #[error("no match in `{0}`")]
    NoMatch(String),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Die {
    pub faces: u8,
}

impl Die {
    fn new(faces: u8) -> Self {
        assert!(faces > 0, "die cannot have 0 faces");
        Self { faces }
    }

    pub fn roll(&self) -> u8 {
        self.roll_with(&mut rand::thread_rng())
    }

    fn roll_with(&self, rng: &mut impl Rng) -> u8 {
        let range = 1..=self.faces;
        range.choose(rng).expect("should have at least one face")
    }

    pub fn d20() -> Self {
        Self::new(20)
    }

    fn min(&self) -> u8 {
        1
    }

    pub fn max(&self) -> u8 {
        self.faces
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Dice {
    vec: Vec<Die>,
    index: usize,
}

impl Dice {
    pub fn new(count: usize, faces: u8) -> Self {
        let vec = vec![Die::new(faces); count.into()];
        Self { vec, index: 0 }
    }

    pub fn roll(&self, rng: StdRng) -> Roll<Self> {
        Roll::new(self.clone(), rng)
    }

    pub fn len(&self) -> usize {
        ExactSizeIterator::len(self)
    }

    #[instrument]
    pub fn lowest_roll(&self) -> usize {
        debug!(len = ?self.len());
        self.len()
    }

    #[instrument]
    pub fn highest_roll(&self) -> usize {
        let highest = self.clone().fold(0, |sum, die| sum + die.max());

        debug!(highest);

        highest as usize
    }
}

impl Iterator for Dice {
    type Item = Die;

    fn next(&mut self) -> Option<Self::Item> {
        let item: Option<&Die> = self.vec.get(self.index);
        self.index += 1;
        item.copied()
    }
}

impl ExactSizeIterator for Dice {
    fn len(&self) -> usize {
        self.vec.len()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Roll<It: Iterator<Item = Die>> {
    iter: It,
    rng: StdRng,
}

impl<It: Iterator<Item = Die>> Roll<It> {
    fn new(iter: It, rng: StdRng) -> Self {
        Self { iter, rng }
    }

    fn total(self, extra: i8) -> isize {
        self.sum::<u8>() as isize + extra as isize
    }
}

impl<It: Iterator<Item = Die>> Iterator for Roll<It> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        let die = self.iter.next();
        die.map(|die| die.roll_with(&mut self.rng))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DiceRoll {
    pub dice: Dice,
    pub extra: i8,
    rng: StdRng,
}

impl DiceRoll {
    pub fn new(count: usize, faces: u8, extra: i8) -> Result<Self, DiceRollError> {
        let dice = Dice::new(count, faces);

        let seed: [u8; 32] = rand::random();
        let rng = StdRng::from_seed(seed);

        let new = Self { dice, extra, rng };
        Ok(new)
    }

    pub fn rolls(&self) -> Roll<Dice> {
        self.dice.roll(self.rng.clone())
    }

    pub fn total(&self) -> isize {
        let sum = self.rolls().sum::<u8>() as isize;
        sum + self.extra as isize
    }

    #[instrument]
    pub fn parse(text: &str) -> Result<Self, DiceRollError> {
        let regex = Regex::new(r"([0-9]*)d([0-9]+)\s*(?:(\+|-)\s*([0-9]+))?")
            .expect("hard-coded regex should be valid");

        let roll = regex
            .captures(text)
            .map(|caps| {
                trace!(?caps);

                let count = caps
                    .get(1)
                    .map_or(Ok(1), |mat| mat.as_str().parse())
                    .unwrap_or_default();
                trace!(?count);
                let faces: u8 = caps
                    .get(2)
                    .ok_or(DiceRollError::NoFaces)?
                    .as_str()
                    .parse()
                    .unwrap();
                trace!(?faces);

                let extra_unsigned = caps.get(4).map(|mat| {
                    let int = mat
                        .as_str()
                        .parse::<i8>()
                        .map_err(|_| DiceRollError::InvalidExtra(mat.as_str().to_owned()));

                    int.unwrap_or_default()
                });
                trace!(?extra_unsigned);

                let extra_sign = caps.get(3).map_or("", |mat| mat.as_str());

                let extra = match extra_sign {
                    "+" => extra_unsigned,
                    "-" => extra_unsigned.map(|int| int.neg()),
                    _ => None,
                }
                .unwrap_or_default();
                debug!(?extra);

                DiceRoll::new(count, faces, extra)
            })
            .ok_or(DiceRollError::NoMatch(text.to_owned()))?;

        debug!(?roll);

        roll
    }

    pub fn min(&self) -> isize {
        self.dice.lowest_roll() as isize + self.extra as isize
    }

    pub fn max(&self) -> isize {
        let highest = self.dice.highest_roll() as isize;
        highest + self.extra as isize
    }
}

mod tests {
    #![allow(unused_imports)]
    use tracing::trace;
    use tracing_test::traced_test;

    use crate::functions::misc::DiceRoll;

    use super::Die;

    macro_rules! test_parse {
        ($name:ident: $text:expr => $parsed:expr$(,)?) => {
            #[test]
            #[traced_test]
            fn $name() {
                tracing::debug!("{:?}", super::DiceRoll::parse($text));
                tracing::debug!("{:?}", $parsed);

                // super dumb fix for broken tests
                pretty_assertions::assert_eq!(
                    format!("{:?}", super::DiceRoll::parse($text)),
                    format!("{:?}", $parsed)
                )
            }
        };

        ($name:ident: $text:expr => $parsed:expr, $($names:ident: $texts:expr => $parseds:expr),+$(,)?) => {
            test_parse!($name: $text => $parsed);
            test_parse! { $($names: $texts => $parseds),+ }
        };
    }

    test_parse! {
        two_d_ten: "2d10" => DiceRoll::new(2, 10, 0),
        d_twenty: "d20" => DiceRoll::new(1, 20, 0),
        d_six_plus_three: "d6+3" => DiceRoll::new(1, 6, 3),
        two_d_four_minus_two: "2d4-2" => DiceRoll::new(2, 4, -2)
    }

    #[test]
    fn roll_die() {
        let die = Die::d20();
        let range = 1..=20;
        let mut rng = rand::thread_rng();

        for _ in 1..1000 {
            let rolled = die.roll_with(&mut rng);
            assert!(range.contains(&rolled))
        }
    }

    #[test]
    #[traced_test]
    fn rolls_sensible() {
        let roll = DiceRoll::parse("2d20").expect("hard-coded");
        let range = 2..=40;

        for _ in 1..1000 {
            let rolls = roll.rolls();
            let sum: u8 = rolls.clone().sum();
            trace!(sum, ?rolls);
            assert!(range.contains(&sum))
        }
    }

    #[test]
    #[traced_test]
    fn rolls_sum_sensible() {
        let roll = DiceRoll::parse("2d20+4").expect("hard-coded");
        let range = 6..=44;
        let extra = roll.extra;

        for _ in 1..2 {
            let roll = roll.clone();
            let sum = roll.total();
            let rolls = roll.rolls();
            trace!(sum, ?rolls, extra);
            assert!(range.contains(&sum))
        }
    }
}
