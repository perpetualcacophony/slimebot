pub mod poise;
pub use poise::Context;

pub mod serenity;

//pub use serenity::AsDiscordId;

use crate::errors::Error;

pub mod format_duration;

pub type Result<T> = std::result::Result<T, Error>;
