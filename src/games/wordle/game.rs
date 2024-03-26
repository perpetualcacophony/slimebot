use poise::serenity_prelude::{EditMessage, Message, UserId};

use crate::{games::wordle::core::AsEmoji, Context};

use super::{
    core::Guess,
    puzzle::{DailyPuzzle, Puzzle},
};

pub struct Game<'a> {
    owner: UserId,
    puzzle: Puzzle,
    guesses: Vec<Guess>,
    ctx: Context<'a>,
    msg: &'a mut Message,
}

impl Game<'_> {
    pub fn count_guesses(&self) -> usize {
        self.guesses.len()
    }

    pub fn title(&self) -> String {
        let puzzle_title = match self.puzzle {
            Puzzle::Random(..) => "random wordle".to_owned(),
            Puzzle::Daily(DailyPuzzle { number, .. }) => format!("wordle {number}"),
        };

        format!("{puzzle_title} {}/6", self.count_guesses())
    }

    pub fn content(&self) -> String {
        format!("{}\n{}", self.title(), self.guesses.as_emoji())
    }

    pub async fn update_message(&mut self) {
        self.msg
            .edit(self.ctx, EditMessage::new().content(self.content()))
            .await
            .unwrap();
    }
}
