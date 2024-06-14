use std::{fmt::Display, iter::Sum, ops::Neg};

use rand::{rngs::StdRng, seq::IteratorRandom, Rng, SeedableRng};
use regex::Regex;

use tracing::{debug, instrument, trace};

use crate::errors::DiceRollError;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Die {
    pub faces: isize,
}

impl Die {
    fn new(faces: isize) -> Self {
        assert!(faces > 0, "die cannot have 0 faces");
        Self { faces }
    }

    fn as_rolled(&self, value: isize) -> RolledDie {
        RolledDie { die: *self, value }
    }

    // convenience version of [`roll_with`] that doesn't use a cached Rng
    pub fn roll(&self) -> RolledDie {
        self.roll_with(&mut rand::thread_rng())
    }

    fn roll_with(&self, rng: &mut impl Rng) -> RolledDie {
        let range = 1..=self.faces;
        let value = range.choose(rng).expect("should have at least one face");
        self.as_rolled(value)
    }

    pub const fn min(&self) -> isize {
        1
    }

    pub fn max(&self) -> isize {
        self.faces
    }
}

impl Default for Die {
    fn default() -> Self {
        Self { faces: 1 }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct RolledDie {
    die: Die,
    value: isize,
}

impl RolledDie {
    fn is_max(&self) -> bool {
        // doesn't make any sense for a d1 or d2 to have a max or min
        self.value == self.die.max() && self.die.faces != 1 && self.die.faces != 2
    }

    fn is_min(&self) -> bool {
        // doesn't make any sense for a d1 or d2 to have a max or min
        self.value == self.die.min() && self.die.faces != 1 && self.die.faces != 2
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Dice {
    vec: Vec<Die>,
    index: usize,
}

impl Dice {
    pub fn new(count: usize, faces: isize) -> Self {
        let vec = vec![Die::new(faces); count];
        Self { vec, index: 0 }
    }

    pub fn roll(&self, rng: StdRng) -> Roll<Self> {
        Roll::new(self.clone(), rng)
    }

    pub fn len(&self) -> isize {
        ExactSizeIterator::len(self) as isize
    }

    #[instrument]
    pub fn lowest_roll(&self) -> isize {
        debug!(len = ?self.len());
        self.len()
    }

    #[instrument]
    pub fn highest_roll(&self) -> isize {
        let highest = self.clone().fold(0, |sum, die| sum + die.max());

        debug!(highest);

        highest
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
}

impl<It: Iterator<Item = Die>> Iterator for Roll<It> {
    type Item = RolledDie;

    fn next(&mut self) -> Option<Self::Item> {
        let die = self.iter.next();
        die.map(|die| die.roll_with(&mut self.rng))
    }
}

impl Sum<RolledDie> for isize {
    fn sum<I: Iterator<Item = RolledDie>>(iter: I) -> Self {
        iter.map(|roll| roll.value).sum()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DiceRoll {
    pub dice: Dice,
    pub extra: isize,
    rng: StdRng,
}

impl DiceRoll {
    pub fn new(count: usize, faces: isize, extra: isize) -> Result<Self, DiceRollError> {
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
        let sum = self.rolls().sum::<isize>();
        sum + self.extra
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
                let faces: isize = caps
                    .get(2)
                    .ok_or(DiceRollError::NoFaces)?
                    .as_str()
                    .parse()
                    .map_err(|_| DiceRollError::NoFaces)?;
                trace!(?faces);

                let extra_unsigned = caps.get(4).map(|mat| {
                    let int = mat
                        .as_str()
                        .parse::<isize>()
                        .map_err(|_| DiceRollError::InvalidExtra(mat.as_str().to_owned()));

                    int.unwrap_or_default()
                });
                trace!(?extra_unsigned);

                let extra_sign = caps.get(3).map(|s| s.as_str());

                let extra = match extra_sign {
                    Some("+") => extra_unsigned,
                    Some("-") => extra_unsigned.map(|int| int.neg()),
                    None => None,
                    _ => {
                        return Err(DiceRollError::InvalidExtraSign(
                            extra_sign.unwrap_or_default().to_owned(),
                        ))
                    }
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
        self.dice.lowest_roll() + self.extra
    }

    pub fn max(&self) -> isize {
        let highest = self.dice.highest_roll();
        highest + self.extra
    }
}

pub struct RollResult {
    dice_roll: DiceRoll,
    rolls: Vec<RolledDie>,
    extra: isize,
    total: isize,
}

impl RollResult {
    #[allow(dead_code)] // used in a macro
    pub fn new(dice_roll: DiceRoll, rolls: impl Into<Vec<RolledDie>>, extra: isize) -> Self {
        let rolls = rolls.into();
        let total = rolls.iter().copied().sum::<isize>() + extra;

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
        let mut text = if (self.is_min() || self.is_max()) && self.rolls[0].die.faces != 1 {
            if self.rolls.len() == 1 && self.rolls[0].die.faces == 2 {
                format!("**{total}**", total = self.total)
            } else {
                format!("**__{total}__**", total = self.total)
            }
        } else {
            format!("**{total}**", total = self.total)
        };

        if self.rolls.len() > 1 || self.extra != 0 {
            text += " (";

            let rolls = self
                .rolls
                .iter()
                .map(|n| {
                    if n.is_max() || n.is_min() {
                        format!("__{num}__", num = n.value)
                    } else {
                        n.value.to_string()
                    }
                })
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

    use super::{DiceRoll, RollResult, RolledDie};

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
        let die = Die::new(20);
        let range = 1..=20;
        let mut rng = rand::thread_rng();

        for _ in 1..1000 {
            let rolled = die.roll_with(&mut rng);
            assert!(range.contains(&rolled.value))
        }
    }

    #[test]
    #[traced_test]
    fn rolls_sensible() {
        let roll = DiceRoll::parse("2d20").expect("hard-coded");
        let range = 2..=40;

        for _ in 1..1000 {
            let rolls = roll.rolls();
            let sum: isize = rolls.clone().sum();
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

                        let rolls = vec![$($rolls),+].into_iter();
                        let dice = rolls.clone().map(|_| super::Die::new($faces));
                        let rolled_dice = rolls.zip(dice).map(|(roll, die)| die.as_rolled(roll));
                        let vec: Vec<super::RolledDie> = rolled_dice.collect();

                        let dice_roll = super::DiceRoll::new(count, $faces, extra).unwrap();
                        let result = super::RollResult::new(dice_roll, vec, extra);

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
            two_d_ten: 10[10, 5] => "**15** (__10__, 5)",
            d_twenty: 20[19] => "**19**",
            three_d_six_plus_three: 6[4, 2, 3] +3 => "**12** (4, 2, 3, +3)",
            three_d_six_minus_three: 6[4, 2, 3] -3 => "**6** (4, 2, 3, -3)",
            d_twenty_plus_three: 20[20] +3 => "**__23__** (__20__, +3)",
            d_twenty_minus_ten: 20[5] -10 => "**-5** (5, -10)",
            nat_twenty: 20[20] => "**__20__**",
            nat_one: 20[1] => "**__1__**",
            nat_one_minus_ten: 20[1] -10 => "**__-9__** (__1__, -10)",
            d_one: 1[1] => "**1**",
            two_d_one: 1[1, 1] => "**2** (1, 1)",
            d_two: 2[1] => "**1**",
            two_d_two: 2[2, 2] => "**__4__** (2, 2)"
        }
    }
}
