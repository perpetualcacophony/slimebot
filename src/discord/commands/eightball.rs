use core::fmt;

use super::CommandResult;
use crate::Context;
use rand::{
    distributions::{
        uniform::{SampleBorrow, SampleUniform},
        Distribution, Standard, WeightedIndex,
    },
    seq::SliceRandom,
    Rng,
};
use tracing::instrument;

macro_rules! create_tone_macros {
    ($($name:ident $tone:expr)+) => {
        $(
            macro_rules! $name {
                ($text:expr) => {
                    $name!($text, 1.0)
                };

                ($text:expr, $weight:expr) => {
                    Answer {
                        tone: $tone,
                        text: $text,
                        weight: $weight,
                    }
                };

                ( $text:expr, $( $texts:expr )+ ) => {
                    $name!($text),
                    $(
                        $name!($texts)
                    )+
                }
            }
        )+
    };
}

create_tone_macros! {
    affirmative AnswerTone::Affirmative
    non_committal AnswerTone::NonCommittal
    negative AnswerTone::Negative
}

macro_rules! create_answer_consts {
    ( $( $answer:expr ),+ ) => {
        const ANSWERS: Answers = Answers(&[
            $( $answer ),+
        ]);
    };
}

create_answer_consts! {
    affirmative!("Yes"),
    non_committal!("Maybe"),
    negative!("No"),
    negative!("No. Banned", 0.1)
}

#[instrument(skip_all)]
#[poise::command(
    slash_command,
    prefix_command,
    rename = "8ball",
    discard_spare_arguments,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
pub async fn eightball(ctx: Context<'_>) -> CommandResult {
    use rand::prelude::thread_rng;

    let answer = ANSWERS.get(&mut thread_rng());
    ctx.reply(answer).await?;

    Ok(())
}

#[derive(PartialEq, Debug, Copy, Clone)]
struct Answer {
    tone: AnswerTone,
    text: &'static str,
    weight: f32,
}

impl Answer {
    fn new(tone: AnswerTone, text: &'static str) -> Self {
        Self {
            tone,
            text,
            weight: 1.0,
        }
    }
}

#[derive(PartialEq, Debug, Copy, Clone)]
enum AnswerTone {
    Affirmative,
    NonCommittal,
    Negative,
}

impl From<Answer> for String {
    fn from(value: Answer) -> Self {
        value.to_string()
    }
}

impl fmt::Display for Answer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.text)
    }
}

struct Answers(&'static [Answer]);

impl Answers {
    fn weighted_dist(&self) -> WeightedIndex<f32> {
        WeightedIndex::new(self.0.iter().map(|ans| ans.weight)).unwrap()
    }

    fn get(&self, rng: &mut impl Rng) -> Answer {
        let weights = self.weighted_dist();
        self.0[weights.sample(rng)]
    }
}

#[cfg(test)]
mod tests {
    mod macros {
        use super::super::{Answer, AnswerTone};
        use pretty_assertions::assert_eq;

        #[test]
        fn affirmative() {
            assert_eq!(
                affirmative!("boop"),
                Answer::new(AnswerTone::Affirmative, "boop")
            );
        }

        #[test]
        fn non_committal() {
            assert_eq!(
                non_committal!("boop"),
                Answer::new(AnswerTone::NonCommittal, "boop")
            );
        }

        #[test]
        fn negative() {
            assert_eq!(
                negative!("boop"),
                Answer::new(AnswerTone::NonCommittal, "boop")
            );
        }
    }
}
