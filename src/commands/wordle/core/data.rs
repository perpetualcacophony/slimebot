use mongodb::Database;

use super::{game::GamesCache, DailyWordles, WordsList};

#[derive(Debug, Clone)]
pub struct WordleData {
    words: kwordle::WordsList<5>,
    wordles: DailyWordles,
    game_data: GamesCache,
}

impl WordleData {
    pub fn new(db: &Database) -> Self {
        let words = kwordle::classic::words_list();
        let wordles = DailyWordles::new(db);
        let game_data = GamesCache::new();

        Self {
            words,
            wordles,
            game_data,
        }
    }

    pub const fn words(&self) -> &kwordle::WordsList<5> {
        &self.words
    }

    pub const fn wordles(&self) -> &DailyWordles {
        &self.wordles
    }

    pub const fn game_data(&self) -> &GamesCache {
        &self.game_data
    }
}
