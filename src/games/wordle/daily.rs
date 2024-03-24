use std::ops::Not;

use chrono::Utc;
use mongodb::{
    bson::doc,
    options::{FindOneOptions, FindOptions},
    Collection, Database,
};
use poise::serenity_prelude::{futures::StreamExt, UserId};
use serde::{Deserialize, Serialize};
use tracing::{debug, instrument, trace};

use super::{core::Word, puzzle, DbResult, GameState};

#[derive(Debug, Clone)]
pub struct DailyWordles {
    collection: Collection<DailyWordle>,
}

impl DailyWordles {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection("daily_wordles"),
        }
    }

    #[instrument(skip_all)]
    pub async fn latest(&self) -> DbResult<Option<DailyWordle>> {
        let daily = self
            .collection
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

    pub async fn new_daily(&self, word: &Word) -> DbResult<DailyWordle> {
        let latest_number = self.latest().await?.map_or(0, |daily| daily.puzzle.number);

        debug!(latest_number);

        let puzzle = puzzle::DailyPuzzle::new(latest_number + 1, word.clone());
        let wordle = DailyWordle::new(puzzle);

        self.collection.insert_one(&wordle, None).await?;

        Ok(wordle)
    }

    pub async fn update(&self, puzzle: u32, game: GameState) -> DbResult<()> {
        let user = mongodb::bson::ser::to_bson(&game.user).expect("implements serialize");
        let game = mongodb::bson::ser::to_bson(&game).expect("implements serialize");

        if self
            .collection
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
            .collection
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
        self.collection
            .find_one(doc! { "puzzle.number": number }, None)
            .await
            .map(|daily| daily.is_some())
    }

    pub async fn find_game(&self, user: UserId, wordle: u32) -> DbResult<Option<GameState>> {
        Ok(self
            .collection
            .find_one(doc! { "puzzle.number": wordle }, None)
            .await?
            .and_then(|daily| daily.user_game(user).cloned()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyWordle {
    pub puzzle: puzzle::DailyPuzzle,
    games: Vec<GameState>,
}

impl DailyWordle {
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

    pub fn user_game(&self, user: UserId) -> Option<&GameState> {
        self.games.iter().find(|game| game.user == user)
    }

    pub fn played_by(&self, user: UserId) -> bool {
        self.user_game(user).is_some()
    }

    pub fn finished_by(&self, user: UserId) -> bool {
        self.user_game(user).is_some_and(|game| game.is_finished())
    }

    pub fn is_playable_for(&self, user: UserId) -> bool {
        self.is_expired().not() && self.finished_by(user).not()
    }

    pub fn in_progress_for(&self, user: UserId) -> bool {
        self.user_game(user).is_some_and(|game| game.in_progress())
    }
}
