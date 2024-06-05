use std::sync::Arc;

use super::{super::core::Guesses, Puzzle};
use poise::serenity_prelude::{ChannelId, MessageId};

#[derive(Clone, Debug)]
pub struct GameData {
    pub puzzle: Arc<Puzzle>,
    pub guesses: Guesses,
    pub channel_id: ChannelId,
    pub message_id: MessageId,
}
