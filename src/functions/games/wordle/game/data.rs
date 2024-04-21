use poise::serenity_prelude::{ChannelId, MessageId};

use crate::functions::games::wordle::core::GuessesRecord;

#[derive(Clone, Debug)]
pub struct GameData {
    pub guesses: GuessesRecord,
    pub channel_id: ChannelId,
    pub message_id: MessageId,
}
