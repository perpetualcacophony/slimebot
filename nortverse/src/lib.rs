#![feature(try_blocks)]
#![feature(duration_constructors)]

mod data;

mod error;
pub use error::NortverseError as Error;

mod comic;

mod client;
pub use client::Nortverse;

mod response;
