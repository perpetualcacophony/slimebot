use std::{borrow::Cow, ops::Not};

use poise::serenity_prelude::UserId;
use serde::{Deserialize, Serialize};

use crate::functions::games::wordle::core::{guess::GuessSlice, AsEmoji, GuessesRecord};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameRecord {
    pub user: UserId,
    guesses: GuessesRecord,
    pub num_guesses: usize,
    finished: bool,
    solved: bool,
}

impl GameRecord {
    pub fn new(owner: UserId, guesses: impl GuessSlice, finished: bool) -> Self {
        let count = guesses.count();
        let solved = guesses.last_is_solved();

        Self {
            user: owner,
            guesses: guesses.to_record(),
            num_guesses: count,
            finished,
            solved,
        }
    }

    pub fn is_finished(&self) -> bool {
        self.finished
    }

    pub fn in_progress(&self) -> bool {
        self.is_finished().not()
    }
}

impl AsEmoji for GameRecord {
    fn as_emoji(&self) -> Cow<str> {
        self.guesses.as_emoji()
    }

    fn emoji_with_letters(&self) -> String {
        self.guesses.emoji_with_letters()
    }

    fn emoji_with_letters_spaced(&self) -> String {
        self.guesses.emoji_with_letters_spaced()
    }
}
