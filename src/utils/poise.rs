use crate::Data;

pub type Context<'a> = poise::Context<'a, Data, crate::Error>;
