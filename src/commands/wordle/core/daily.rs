use std::ops::Not;

use chrono::Utc;
use mongodb::{
    bson::doc,
    options::{FindOneOptions, FindOptions},
    Collection, Database,
};
use poise::serenity_prelude::{
    futures::{Stream, StreamExt, TryStreamExt},
    UserId,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, instrument, trace};

use super::{puzzle, DbResult, GameRecord};

#[derive(Debug, Clone)]
pub struct DailyWordles {
    collection: Collection<PartialDailyWordle>,
    words_list: kwordle::WordsList,
}

impl DailyWordles {
    async fn find_one(
        &self,
        filter: impl Into<Option<mongodb::bson::Document>>,
        options: impl Into<Option<FindOneOptions>>,
    ) -> DbResult<Option<DailyWordle>> {
        Ok(self
            .collection
            .find_one(filter, options)
            .await?
            .and_then(|partial| DailyWordle::from_partial(partial, &self.words_list)))
    }

    async fn find(
        &self,
        filter: impl Into<Option<mongodb::bson::Document>>,
        options: impl Into<Option<FindOptions>>,
    ) -> DbResult<impl Stream<Item = DbResult<DailyWordle>> + '_> {
        Ok(self.collection.find(filter, options).await?.map(|res| {
            res.map(|partial| DailyWordle::from_partial(partial, &self.words_list).unwrap())
        }))
    }

    pub fn new(db: &Database, words: &kwordle::WordsList) -> Self {
        Self {
            collection: db.collection("daily_wordles"),
            words_list: words.clone(),
        }
    }

    #[instrument(skip_all)]
    pub async fn latest(&self) -> DbResult<Option<DailyWordle>> {
        let daily = self
            .find_one(
                None,
                FindOneOptions::builder()
                    .sort(doc! { "puzzle.number": -1 })
                    .build(),
            )
            .await?;

        debug!(?daily);

        let puzzle = daily.clone().map(|d| d.puzzle.number);

        debug!(?puzzle);

        Ok(daily)
    }

    pub async fn latest_not_expired(&self) -> DbResult<Option<DailyWordle>> {
        Ok(self
            .latest()
            .await?
            .filter(|daily| daily.is_expired().not()))
    }

    pub async fn refresh(&self, words: &kwordle::WordsList) -> DbResult<Option<DailyWordle>> {
        let new_word = words.answers.random();

        if let Some(latest) = self.latest_not_expired().await? {
            if latest.is_old() {
                return self.new_daily(&new_word).await.map(Some);
            }
        } else {
            return self.new_daily(&new_word).await.map(Some);
        }

        Ok(None)
    }

    pub async fn new_daily(&self, word: &kwordle::Word<5>) -> DbResult<DailyWordle> {
        let latest_number = self.latest().await?.map_or(0, |daily| daily.puzzle.number);

        debug!(latest_number);

        let puzzle = puzzle::DailyPuzzle::new(latest_number + 1, word.clone());
        let wordle = DailyWordle::new(puzzle);

        self.collection
            .insert_one(&wordle.clone().into_partial(), None)
            .await?;

        Ok(wordle)
    }

    pub async fn update(&self, puzzle: u32, game: GameRecord) -> DbResult<()> {
        let user = mongodb::bson::ser::to_bson(&game.user).expect("implements serialize");
        let game = mongodb::bson::ser::to_bson(&game).expect("implements serialize");

        if self
            .find_one(
                doc! {
                    "puzzle.number": puzzle,
                    "games": { "$elemMatch": { "user": &user } }
                },
                None,
            )
            .await?
            .is_some()
        {
            trace!("game exists in db");

            self.collection
                .update_one(
                    doc! {
                        "puzzle.number": puzzle,
                        "games": { "$elemMatch": { "user": &user } }
                    },
                    doc! { "$set": { "games.$": game } },
                    None,
                )
                .await?;
        } else {
            trace!("game does not exist in db");

            self.collection
                .update_one(
                    doc! { "puzzle.number": puzzle },
                    doc! { "$addToSet": {
                        "games": game
                    } },
                    None,
                )
                .await?;
        }

        Ok(())
    }

    async fn not_expired(&self) -> DbResult<Vec<DailyWordle>> {
        let mut vec = Vec::with_capacity(2);

        let mut cursor = self
            .find(
                None,
                FindOptions::builder()
                    // anything beyond the first 2 wordles will always be expired
                    .sort(doc! { "puzzle.number":-1 })
                    .limit(2)
                    .build(),
            )
            .await?;

        while let Some(daily) = cursor.next().await {
            if daily.as_ref().is_ok_and(|daily| daily.is_expired().not()) {
                vec.push(daily?);
            }
        }

        Ok(vec)
    }

    pub async fn playable_for(&self, user: UserId) -> DbResult<impl Iterator<Item = DailyWordle>> {
        Ok(self
            .not_expired()
            .await?
            .into_iter()
            .filter(move |daily| daily.is_playable_for(user)))
    }

    pub async fn wordle_exists(&self, number: u32) -> DbResult<bool> {
        self.find_one(doc! { "puzzle.number": number }, None)
            .await
            .map(|daily| daily.is_some())
    }

    pub async fn find_game(&self, user: UserId, wordle: u32) -> DbResult<Option<GameRecord>> {
        Ok(self
            .find_one(doc! { "puzzle.number": wordle }, None)
            .await?
            .and_then(|daily| daily.user_game(user).cloned()))
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DailyWordle {
    pub puzzle: puzzle::DailyPuzzle,
    games: Vec<GameRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PartialDailyWordle {
    pub puzzle: puzzle::PartialDailyPuzzle,
    games: Vec<GameRecord>,
}

impl DailyWordle {
    fn from_partial(partial: PartialDailyWordle, list: &kwordle::WordsList) -> Option<Self> {
        Some(Self {
            puzzle: puzzle::DailyPuzzle::from_partial(partial.puzzle, list)?,
            games: partial.games,
        })
    }

    fn into_partial(self) -> PartialDailyWordle {
        PartialDailyWordle {
            puzzle: self.puzzle.into_partial(),
            games: self.games,
        }
    }

    fn new(puzzle: puzzle::DailyPuzzle) -> Self {
        Self {
            puzzle,
            games: Vec::new(),
        }
    }

    pub fn age_hours(&self) -> i64 {
        let age = Utc::now() - self.puzzle.started;
        age.num_hours()
    }

    pub fn is_recent(&self) -> bool {
        self.age_hours() < 24
    }

    pub fn is_old(&self) -> bool {
        self.age_hours() < 48 && !self.is_recent()
    }

    pub fn is_expired(&self) -> bool {
        self.age_hours() >= 48
    }

    pub fn user_game(&self, user: UserId) -> Option<&GameRecord> {
        self.games.iter().find(|game| game.user == user)
    }

    #[allow(dead_code)]
    pub fn played_by(&self, user: UserId) -> bool {
        self.user_game(user).is_some()
    }

    pub fn finished_by(&self, user: UserId) -> bool {
        self.user_game(user).is_some_and(|game| game.is_finished())
    }

    pub fn is_playable_for(&self, user: UserId) -> bool {
        self.is_expired().not() && self.finished_by(user).not()
    }

    #[allow(dead_code)]
    pub fn in_progress_for(&self, user: UserId) -> bool {
        self.user_game(user).is_some_and(|game| game.in_progress())
    }
}

#[cfg(test)]
mod tests {
    use super::DailyWordle;
    use pretty_assertions::assert_str_eq;

    const DAILY_WORDLE_JSON: &str = include_str!("./tests/daily_wordle.json");

    #[test]
    fn deserialize() {
        let words = kwordle::classic::words_list();

        DailyWordle::from_partial(
            serde_json::from_str(DAILY_WORDLE_JSON).expect("should be valid json"),
            &words,
        )
        .expect("should be valid DailyWordle");
    }

    #[test]
    fn serialize_stable() {
        let words = kwordle::classic::words_list();

        let daily_wordle = DailyWordle::from_partial(
            serde_json::from_str(DAILY_WORDLE_JSON).expect("should be valid json"),
            &words,
        )
        .expect("should be valid DailyWordle");

        let serialized =
            serde_json::to_string_pretty(&daily_wordle).expect("should serialize properly");

        assert_str_eq!(serialized, DAILY_WORDLE_JSON)
    }
}
