use std::sync::Arc;

use poise::serenity_prelude::{ChannelId, MessageId};

use crate::functions::games::wordle::{core::Guesses, Puzzle};

#[derive(Clone, Debug)]
pub struct GameData {
    pub puzzle: Arc<Puzzle>,
    pub guesses: Guesses,
    pub channel_id: ChannelId,
    pub message_id: MessageId,
}
