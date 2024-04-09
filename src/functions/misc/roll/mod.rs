use std::{fmt::Display, ops::Neg};

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

    pub fn max(&self) -> u8 {
        self.faces
    }
}

impl Default for Die {
    fn default() -> Self {
        Self { faces: 1 }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Dice {
    vec: Vec<Die>,
    index: usize,
}

impl Dice {
    pub fn new(count: usize, faces: u8) -> Self {
        let vec = vec![Die::new(faces); count];
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

    pub fn result(self) -> RollResult {
        let rolls = self.rolls().collect();
        let extra = self.extra;
        let total = self.total();

        RollResult {
            dice_roll: self,
            rolls,
            extra,
            total,
        }
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
                    .unwrap_or(1);
                trace!(?count);
                let faces: u8 = caps
                    .get(2)
                    .ok_or(DiceRollError::NoFaces)?
                    .as_str()
                    .parse()
                    .map_err(|_| DiceRollError::NoFaces)?;
                trace!(?faces);

                let extra_unsigned = caps.get(4).map(|mat| {
                    let int = mat
                        .as_str()
                        .parse::<i8>()
                        .map_err(|_| DiceRollError::InvalidExtra(mat.as_str().to_owned()));

                    int.unwrap_or_default()
                });
                trace!(?extra_unsigned);

                let extra_sign = caps
                    .get(3)
                    .ok_or(DiceRollError::InvalidExtraSign(String::new()))
                    .map(|mat| mat.as_str())?;

                let extra = match extra_sign {
                    "+" => extra_unsigned,
                    "-" => extra_unsigned.map(|int| int.neg()),
                    _ => return Err(DiceRollError::InvalidExtraSign(extra_sign.to_owned())),
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

pub struct RollResult {
    dice_roll: DiceRoll,
    rolls: Vec<u8>,
    extra: i8,
    total: isize,
}

impl RollResult {
    fn new(dice_roll: DiceRoll, rolls: impl Into<Vec<u8>>, extra: i8) -> Self {
        let rolls = rolls.into();
        let total = rolls.iter().sum::<u8>() as isize + extra as isize;

        Self {
            dice_roll,
            rolls,
            extra,
            total,
        }
    }

    fn is_min(&self) -> bool {
        self.total == self.dice_roll.min()
    }

    fn is_max(&self) -> bool {
        self.total == self.dice_roll.max()
    }
}

impl Display for RollResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut text = if self.is_min() || self.is_max() {
            format!("**__{total}__**", total = self.total)
        } else {
            format!("**{total}**", total = self.total)
        };

        if self.rolls.len() > 1 || self.extra != 0 {
            text += " (";

            let rolls = self
                .rolls
                .iter()
                .map(|n| n.to_string())
                .collect::<Vec<_>>()
                .join(", ");

            text += &rolls;

            if self.extra != 0 {
                let extra_text = if self.extra.is_positive() {
                    format!(", +{num}", num = self.extra)
                } else {
                    format!(", {num}", num = self.extra)
                };

                text += &extra_text;
            }

            text += ")";
        }

        f.write_str(&text)
    }
}

mod tests {
    #![allow(unused_imports)]
    use tracing::trace;
    use tracing_test::traced_test;

    use crate::functions::misc::{DiceRoll, RollResult};

    use super::Die;

    mod parse {
        use super::DiceRoll;

        macro_rules! generate_tests {
            ($($name:ident: $text:expr => $parsed:expr),+) => {
                paste::paste! {
                    $(
                        #[test]
                        #[tracing_test::traced_test]
                        fn [<parse_ $name>]() {
                            tracing::debug!("{:?}", super::DiceRoll::parse($text));
                            tracing::debug!("{:?}", $parsed);

                            // super dumb fix for broken tests
                            pretty_assertions::assert_eq!(
                                format!("{:?}", super::DiceRoll::parse($text)),
                                format!("{:?}", $parsed)
                            )
                        }
                    )+
                }
            };
        }

        generate_tests! {
            two_d_ten: "2d10" => DiceRoll::new(2, 10, 0),
            d_twenty: "d20" => DiceRoll::new(1, 20, 0),
            d_six_plus_three: "d6+3" => DiceRoll::new(1, 6, 3),
            two_d_four_minus_two: "2d4-2" => DiceRoll::new(2, 4, -2)
        }
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

    mod format_result {
        #[allow(unused_macros)]
        macro_rules! or_else {
            ( ; $else:expr) => {
                $else
            };
            ($target:literal ; $else:expr) => {
                $target
            };
        }

        macro_rules! generate_tests {
            ($($name:ident: $faces:literal[$($rolls:literal),+] $($(+)?$extra:literal)? => $formatted:literal),+) => {
                $(
                    #[test]
                    #[tracing_test::traced_test]
                    fn $name() {
                        //tracing::debug!("{:?}", super::DiceRoll::parse($text));
                        //tracing::debug!("{:?}", $parsed);

                        let extra = or_else!($($extra)? ; 0);
                        let count = ${count($rolls)};

                        let dice_roll = super::DiceRoll::new(count, $faces, extra).unwrap();
                        let result = super::RollResult::new(dice_roll, [$($rolls),+], extra);

                        // super dumb fix for broken tests
                        pretty_assertions::assert_eq!(
                            format!("{}", result),
                            $formatted.to_string()
                        )
                    }
                )+
            };
        }

        generate_tests! {
            two_d_ten: 10[10, 15] => "**25** (10, 15)",
            d_twenty: 20[19] => "**19**",
            three_d_six_plus_three: 6[4, 2, 3] +3 => "**12** (4, 2, 3, +3)",
            three_d_six_minus_three: 6[4, 2, 3] -3 => "**6** (4, 2, 3, -3)",
            d_twenty_plus_three: 20[20] +3 => "**__23__** (20, +3)",
            d_twenty_minus_ten: 20[5] -10 => "**-5** (5, -10)",
            nat_twenty: 20[20] => "**__20__**",
            nat_one: 20[1] => "**__1__**",
            nat_one_minus_ten: 20[1] -10 => "**__-9__** (1, -10)"
        }
    }
}
