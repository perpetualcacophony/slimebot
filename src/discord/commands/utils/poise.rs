use crate::Data;
use crate::Error;

pub type Command = poise::Command<Data, Error>;
pub type CommandResult = Result<(), Error>;
