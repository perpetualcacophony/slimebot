use thiserror::Error;

use super::DailyPuzzle;

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
    #[error("only one puzzle in database, so no 'previous puzzle' exists")]
    OnlyOnePuzzle,
    #[error("mongodb error: {0:#}")]
    MongoDb(#[from] mongodb::error::Error),
}
