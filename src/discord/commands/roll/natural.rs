use std::{
    iter::Sum,
    num::{NonZeroI8, ParseIntError, TryFromIntError},
    str::FromStr,
};

use serde::Deserialize;
use thiserror::Error;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
pub struct NaturalI8(NonZeroI8);

impl NaturalI8 {
    pub fn new(value: NonZeroI8) -> Result<Self, NaturalI8Error> {
        value.try_into()
    }

    pub fn get(&self) -> i8 {
        self.get_non_zero().get()
    }

    pub fn get_non_zero(&self) -> NonZeroI8 {
        self.0
    }

    pub fn min() -> Self {
        Self::one()
    }
}

pub use natural_consts::NaturalI8Constants;
mod natural_consts {
    use super::NaturalI8;
    use std::num::NonZeroI8;

    macro_rules! natural_const {
        ($name:ident: $num:expr$(,)?) => {
            fn $name() -> NaturalI8 {
                NaturalI8::new(
                    NonZeroI8::new(1).expect(format!("{} != 0", $num).as_str())
                ).expect(format!("{} >= 1", $num).as_str())
            }
        };

        ($name:ident: $num:expr, $($names:ident: $nums:expr),+$(,)?) => {
            natural_const!($name: $num);
            natural_const! { $($names: $nums),+ }
        };
    }

    pub trait NaturalI8Constants {
        natural_const! {
            one: 1,
            twenty: 20,
            one_hundred: 100,
        }
    }

    impl NaturalI8Constants for NaturalI8 {}
}

impl std::iter::Sum for NaturalI8 {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.map(|natural| natural.get())
            .sum::<i8>()
            .try_into()
            .expect("sum of naturals must be natural")
    }
}

impl Default for NaturalI8 {
    fn default() -> Self {
        Self::min()
    }
}

impl std::fmt::Debug for NaturalI8 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl TryFrom<NonZeroI8> for NaturalI8 {
    type Error = NaturalI8Error;

    fn try_from(value: NonZeroI8) -> Result<Self, Self::Error> {
        if value.get() >= 1 {
            Ok(Self(value))
        } else {
            Err(NaturalI8Error::ValueNegative(value))
        }
    }
}

impl TryFrom<i8> for NaturalI8 {
    type Error = NaturalI8Error;

    fn try_from(value: i8) -> Result<Self, Self::Error> {
        let non_zero: NonZeroI8 = value.try_into()?;

        if non_zero.get() >= 1 {
            Ok(Self(non_zero))
        } else {
            Err(NaturalI8Error::ValueNegative(non_zero))
        }
    }
}

impl Sum<NaturalI8> for i8 {
    fn sum<I: Iterator<Item = NaturalI8>>(iter: I) -> Self {
        iter.map(|natural| natural.get()).sum()
    }
}

impl Sum<NaturalI8> for i16 {
    fn sum<I: Iterator<Item = NaturalI8>>(iter: I) -> Self {
        iter.map(|natural| natural.get() as i16).sum()
    }
}

impl FromStr for NaturalI8 {
    type Err = NaturalI8Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let non_zero: NonZeroI8 = s.parse()?;

        if non_zero.get() >= 1 {
            Ok(Self(non_zero))
        } else {
            Err(NaturalI8Error::ValueNegative(non_zero))
        }
    }
}

impl From<NaturalI8> for usize {
    fn from(val: NaturalI8) -> Self {
        val.get().try_into().expect("usize > 0 && NaturalI8 > 0")
    }
}

impl ToString for NaturalI8 {
    fn to_string(&self) -> String {
        self.get().to_string()
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum NaturalI8Error {
    #[error("parsed value as zero")]
    ParsedZero(#[from] ParseIntError),
    #[error("value cannot be zero")]
    TryFromZero(#[from] TryFromIntError),
    #[error("value `{0}` is negative")]
    ValueNegative(NonZeroI8),
}

impl From<NaturalI8> for i16 {
    fn from(value: NaturalI8) -> Self {
        value.get().into()
    }
}
