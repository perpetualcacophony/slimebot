use super::players;
use players::Players;

#[derive(Debug)]
pub struct Data<B = players::MongoDb> {
    pub(crate) players: Players<B>,
}

impl<B> Data<B> {
    pub(crate) fn players(&self) -> Players<B> {
        self.players.clone()
    }
}

impl Data<players::HashMap> {
    pub fn new_map() -> Self {
        Self {
            players: Players::<players::HashMap>::new(),
        }
    }
}

impl Data<players::MongoDb> {
    pub fn new_mongodb(db: &mongodb::Database) -> Self {
        Self {
            players: Players::<players::MongoDb>::new(db.collection("minecraft")),
        }
    }
}

impl<B> Clone for Data<B> {
    fn clone(&self) -> Self {
        Self {
            players: self.players.clone(),
        }
    }
}
