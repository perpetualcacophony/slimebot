use std::{num::NonZeroI8, ops::Neg};

use rand::{rngs::StdRng, seq::IteratorRandom, Rng, SeedableRng};
use regex::Regex;

use thiserror::Error;
use tracing::{debug, instrument, trace};

pub mod natural;
use natural::{NaturalI8, NaturalI8Constants, NaturalI8Error};
#[derive(Debug, Error, PartialEq)]
pub enum DiceRollError {
    #[error(transparent)]
    InvalidNumber(#[from] NaturalI8Error),
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
    pub faces: NaturalI8,
}

impl Die {
    fn new(faces: NaturalI8) -> Self {
        Self { faces }
    }

    pub fn roll(&self) -> NaturalI8 {
        self.roll_with(&mut rand::thread_rng())
    }

    fn roll_with(&self, rng: &mut impl Rng) -> NaturalI8 {
        let range = 1..=self.faces.get();

        range
            .choose(rng)
            .expect("should have at least one face")
            .try_into()
            .expect("faces is a valid NaturalI8")
    }

    pub fn d20() -> Self {
        Self::new(NaturalI8::twenty())
    }

    fn min(&self) -> NaturalI8 {
        NaturalI8::min()
    }

    pub fn max(&self) -> NaturalI8 {
        self.faces
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Dice {
    vec: Vec<Die>,
    index: usize,
}

impl Dice {
    pub fn new(count: NaturalI8, faces: NaturalI8) -> Self {
        let vec = vec![Die::new(faces); count.into()];

        Self { vec, index: 0 }
    }

    pub fn roll(&self, rng: StdRng) -> Roll<Self> {
        Roll::new(self.clone(), rng)
    }

    pub fn len(&self) -> NaturalI8 {
        ExactSizeIterator::len(self)
            .try_into()
            .expect("number of dice should not be 0")
    }

    #[instrument]
    pub fn lowest_roll(&self) -> NaturalI8 {
        debug!(len = ?self.len());
        self.len()
    }

    #[instrument]
    pub fn highest_roll(&self) -> i16 {
        let highest = self
            .clone()
            .fold(0, |sum, die| sum + die.max().get() as i16);

        debug!(highest);

        highest
    }
}

impl TryFrom<usize> for NaturalI8 {
    type Error = NaturalI8Error;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        let int: i8 = value.try_into()?;
        let non_zero: NonZeroI8 = int.try_into()?;

        non_zero.try_into()
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

    fn total(self, extra: i8) -> i16 {
        self.sum::<i16>() + extra as i16
    }
}

impl<It: Iterator<Item = Die>> Iterator for Roll<It> {
    type Item = NaturalI8;

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
    pub fn new(count: i8, faces: i8, extra: i8) -> Result<Self, DiceRollError> {
        let faces = faces.try_into()?;
        let count = count.try_into()?;

        let dice = Dice::new(count, faces);

        let seed: [u8; 32] = rand::random();
        let rng = StdRng::from_seed(seed);

        let new = Self { dice, extra, rng };
        Ok(new)
    }

    pub fn rolls(&self) -> Roll<Dice> {
        self.dice.roll(self.rng.clone())
    }

    pub fn total(&self) -> i16 {
        let sum = self.rolls().sum::<i16>();
        sum + self.extra as i16
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
                    .map_or(Ok(NaturalI8::default()), |mat| mat.as_str().parse())
                    .unwrap_or_default();
                trace!(?count);
                let faces: NaturalI8 = caps
                    .get(2)
                    .ok_or(DiceRollError::NoFaces)?
                    .as_str()
                    .parse()?;
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

                DiceRoll::new(count.get(), faces.get(), extra)
            })
            .ok_or(DiceRollError::NoMatch(text.to_owned()))?;

        debug!(?roll);

        roll
    }

    pub fn min(&self) -> i16 {
        self.dice.lowest_roll().get() as i16 + self.extra as i16
    }

    pub fn max(&self) -> i16 {
        let highest: i16 = self.dice.highest_roll();
        highest + self.extra as i16
    }
}

mod tests {
    #![allow(unused_imports)]
    use tracing::trace;
    use tracing_test::traced_test;

    use crate::discord::commands::roll::NaturalI8;

    use super::{natural::NaturalI8Constants, DiceRoll, Die};

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
        let range = NaturalI8::one()..=NaturalI8::twenty();
        let mut rng = rand::thread_rng();

        for _ in 1..1000 {
            let rolled: NaturalI8 = die.roll_with(&mut rng);
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
            let sum: i8 = rolls.clone().sum();
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
            let sum: i16 = roll.total();
            let rolls = roll.rolls();
            trace!(sum, ?rolls, extra);
            assert!(range.contains(&sum))
        }
    }
}
