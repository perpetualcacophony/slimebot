use thiserror::Error;

use super::puzzle::DailyPuzzle;

#[derive(Debug, Error)]
pub enum Error {
    #[error("latest puzzle is more than {} hours old", super::PUZZLE_ACTIVE_HOURS)]
    LatestTooOld(DailyPuzzle),
    #[error(
        "puzzle has expired (older than {} hours)",
        super::PUZZLE_ACTIVE_HOURS * 2
    )]
    Expired(DailyPuzzle),
    #[error("no puzzles in database")]
    NoPuzzles,
    #[error("mongodb error: {0:#}")]
    MongoDb(#[from] mongodb::error::Error),
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}
