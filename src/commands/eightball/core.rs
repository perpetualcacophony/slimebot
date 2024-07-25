use rand::{
    distributions::{Distribution, WeightedIndex},
    Rng,
};
use std::fmt;

macro_rules! create_answer_consts {
    (
        affirmative {
            $($text:literal $($weight:literal)?),+
        }

        non_committal {
            $($text2:literal $($weight2:literal)?),+
        }

        negative {
            $($text3:literal $($weight3:literal)?),+
        }
    ) => {
        macro_rules! weight {
            ($value:literal) => { $value };
            () => { 1.0 }
        }

        pub const ANSWERS: Answers = Answers(&[
            $(Answer { tone: AnswerTone::Affirmative, text: $text, weight: weight!($($weight)?)},)+
            $(Answer { tone: AnswerTone::NonCommittal, text: $text2, weight: weight!($($weight2)?)} ,)+
            $(Answer { tone: AnswerTone::Negative, text: $text3, weight: weight!($($weight3)?)} ,)+
        ]);
    }
}

create_answer_consts! {
    affirmative {
        "It is certain",
        "It is decidedly so",
        "Without a doubt",
        "Yes definitely",
        "You may rely on it",
        "As I see it, yes",
        "Most likely",
        "Outlook good",
        "Yes",
        "Signs point to yes",
        "Yep",
        "Mhm"
    }

    non_committal {
        "Reply hazy, try again",
        "Ask again later",
        "Better not tell you now",
        "Cannot predict now",
        "Concentrate and ask again",
        "Ask again in six to eight business weeks"
    }

    negative {
        "Don't count on it",
        "My reply is no",
        "My sources say no",
        "Outlook not so good",
        "Very doubtful",
        "No. Banned" 0.1
    }
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub struct Answer {
    tone: AnswerTone,
    text: &'static str,
    weight: f32,
}

impl Answer {
    #[allow(dead_code)] // here by convention
    fn new(tone: AnswerTone, text: &'static str) -> Self {
        Self {
            tone,
            text,
            weight: 1.0,
        }
    }
}

#[derive(PartialEq, Debug, Copy, Clone, Hash, Eq, PartialOrd, Ord)]
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

pub struct Answers(&'static [Answer]);

impl Answers {
    fn weighted_dist(&self) -> WeightedIndex<f32> {
        WeightedIndex::new(self.0.iter().map(|ans| ans.weight))
            .expect("should have more than 1 answer")
    }

    pub fn get(&self, rng: &mut impl Rng) -> Answer {
        let weights = self.weighted_dist();
        self.0[weights.sample(rng)]
    }
}

#[cfg(test)]
mod tests {
    /// These tests are here to make sure that the magic 8-ball's odds don't change
    /// Or at least, if they do change, it'll pop up in CI
    /// So that the odds don't change dramatically, which isn't breaking but would be. Weird
    mod stability {
        use std::collections::HashMap;

        use super::super::AnswerTone;

        use super::super::ANSWERS;

        fn _factors(num: usize, vec: &mut Vec<usize>) {
            vec.push(num);

            for x in (2..num).rev() {
                if vec.contains(&x) {
                    continue;
                }

                if num % x == 0 {
                    _factors(x, vec);
                }
            }
        }

        fn factors(num: usize) -> Vec<usize> {
            let mut vec = Vec::new();
            _factors(num, &mut vec);
            vec.push(1);
            vec.reverse();
            vec
        }

        fn gcd(nums: &[usize]) -> usize {
            let factors: Vec<Vec<usize>> = nums.iter().map(|n| factors(*n)).collect();
            'outer: for n in factors[0].iter().rev() {
                for list in factors.iter() {
                    if !list.contains(n) {
                        continue 'outer;
                    }
                }

                return *n;
            }

            1
        }

        impl super::super::Answers {
            fn tone_counts(&self) -> HashMap<AnswerTone, usize> {
                let mut map = HashMap::new();

                for answer in self.0 {
                    if let Some(count) = map.get_mut(&answer.tone) {
                        *count += 1;
                    } else {
                        map.insert(answer.tone, 1);
                    }
                }

                map
            }

            #[allow(dead_code)]
            // not sure how needed this is, and it brings in a lot of extra code
            // but it only compiles on test, so it's fiiiiiiine
            fn tone_ratios(&self) -> HashMap<AnswerTone, usize> {
                let counts = self.tone_counts();
                let counts: Vec<usize> = counts.values().copied().collect();
                let gcd = gcd(&counts);
                self.tone_counts()
                    .iter_mut()
                    .map(|(tone, count)| (*tone, *count / gcd))
                    .collect()
            }

            fn tone_weights(&self) -> HashMap<AnswerTone, f32> {
                let mut map = HashMap::new();

                for answer in self.0 {
                    if let Some(total) = map.get_mut(&answer.tone) {
                        *total += answer.weight;
                    } else {
                        map.insert(answer.tone, answer.weight);
                    }
                }

                map
            }
        }

        #[test]
        fn test_factors() {
            std::assert_eq!(factors(10).as_ref(), [1, 2, 5, 10])
        }

        #[test]
        fn greatest_common_denominator() {
            assert_eq!(gcd(&[32, 48]), 16)
        }

        #[test]
        fn tone_counts() {
            let map = ANSWERS.tone_counts();
            let mut tone_counts = map.iter().collect::<Vec<_>>();
            tone_counts.sort();
            let tone_counts = tone_counts
                .iter()
                .map(|(_, count)| count)
                .copied()
                .copied()
                .collect::<Vec<usize>>();

            assert_eq!(tone_counts.as_slice(), &[12, 6, 6])
        }

        #[test]
        fn tone_weights() {
            let map = ANSWERS.tone_weights();
            let mut tone_weights = map.iter().collect::<Vec<_>>();
            tone_weights.sort_by(|a, b| a.0.cmp(b.0));
            let tone_weights = tone_weights
                .iter()
                .map(|(_, count)| count)
                .copied()
                .copied()
                .collect::<Vec<f32>>();

            assert_eq!(tone_weights.as_slice(), &[12.0, 6.0, 5.1])
        }
    }
}
