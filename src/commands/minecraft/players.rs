use poise::serenity_prelude::{self as serenity, futures::StreamExt};

use super::error::ErrorAlreadyClaimed;

pub trait Backend {
    type Error;
    type Result<T> = std::result::Result<T, Self::Error>;

    // read
    async fn player_from_minecraft(&self, username: &str)
        -> Self::Result<Option<serenity::UserId>>;
    async fn player_from_discord(&self, id: serenity::UserId) -> Self::Result<Vec<String>>;

    // update
    async fn add_username(
        &self,
        id: serenity::UserId,
        username: String,
    ) -> Self::Result<Result<(), ErrorAlreadyClaimed>>;
}

#[derive(Debug, Default)]
pub struct Players<Backend> {
    backend: std::sync::Arc<Backend>,
}

impl<B> Clone for Players<B> {
    fn clone(&self) -> Self {
        Self {
            backend: self.backend.clone(),
        }
    }
}

impl<B: Backend> Players<B> {
    pub async fn player_from_minecraft(
        &self,
        username: &str,
    ) -> B::Result<Option<serenity::UserId>> {
        self.backend.player_from_minecraft(username).await
    }

    pub async fn player_from_discord(&self, id: serenity::UserId) -> B::Result<Vec<String>> {
        self.backend.player_from_discord(id).await
    }

    pub async fn add_username(
        &self,
        id: serenity::UserId,
        username: String,
    ) -> B::Result<Result<(), ErrorAlreadyClaimed>> {
        self.backend.add_username(id, username).await
    }
}

impl<B> From<B> for Players<B> {
    fn from(value: B) -> Self {
        Self {
            backend: std::sync::Arc::new(value),
        }
    }
}

pub type HashMap = tokio::sync::RwLock<std::collections::HashMap<String, serenity::UserId>>;

pub type PlayersHashMap = Players<HashMap>;

impl PlayersHashMap {
    pub fn new() -> Self {
        Self::default()
    }
}

pub type PlayersMongoDb = Players<MongoDb>;

impl PlayersMongoDb {
    pub fn new(collection: mongodb::Collection<Record>) -> Self {
        MongoDb::new(collection).into()
    }
}

impl BackendInfallible
    for tokio::sync::RwLock<std::collections::HashMap<String, serenity::UserId>>
{
    async fn player_from_minecraft(&self, username: &str) -> Option<serenity::UserId> {
        let guard = self.read().await;
        guard.get(username).copied()
    }

    async fn player_from_discord(&self, id: serenity::UserId) -> Vec<String> {
        let guard = self.read().await;
        guard
            .iter()
            .filter_map(|(name, val_id)| (*val_id == id).then_some(name.clone()))
            .collect()
    }

    async fn add_username(
        &self,
        id: serenity::UserId,
        username: String,
    ) -> Result<(), ErrorAlreadyClaimed> {
        let mut guard = self.write().await;
        if guard.insert(username.clone(), id).is_some() {
            Err(ErrorAlreadyClaimed::new(id, None, username))
        } else {
            Ok(())
        }
    }
}

trait BackendInfallible {
    async fn player_from_minecraft(&self, username: &str) -> Option<serenity::UserId>;
    async fn player_from_discord(&self, id: serenity::UserId) -> Vec<String>;
    async fn add_username(
        &self,
        id: serenity::UserId,
        username: String,
    ) -> Result<(), ErrorAlreadyClaimed>;
}
impl<B: BackendInfallible> Backend for B {
    type Error = std::convert::Infallible;

    async fn player_from_minecraft(
        &self,
        username: &str,
    ) -> Self::Result<Option<serenity::UserId>> {
        Ok(BackendInfallible::player_from_minecraft(self, username).await)
    }

    async fn player_from_discord(&self, id: serenity::UserId) -> Self::Result<Vec<String>> {
        Ok(BackendInfallible::player_from_discord(self, id).await)
    }

    async fn add_username(
        &self,
        id: serenity::UserId,
        username: String,
    ) -> Self::Result<Result<(), ErrorAlreadyClaimed>> {
        Ok(BackendInfallible::add_username(self, id, username).await)
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct Record {
    minecraft_username: String,
    discord_id: serenity::UserId,
}

#[derive(Debug, Clone)]
pub struct MongoDb {
    collection: mongodb::Collection<Record>,
}

impl MongoDb {
    pub fn new(collection: mongodb::Collection<Record>) -> Self {
        Self { collection }
    }
}

impl Backend for MongoDb {
    type Error = mongodb::error::Error;

    async fn player_from_minecraft(
        &self,
        username: &str,
    ) -> Self::Result<Option<serenity::UserId>> {
        self.collection
            .find_one(mongodb::bson::doc! { "minecraft_username": username }, None)
            .await
            .map(|op| op.map(|record| record.discord_id))
    }

    async fn player_from_discord(&self, id: serenity::UserId) -> Self::Result<Vec<String>> {
        let id = &mongodb::bson::ser::to_bson(&id).expect("UserId implements Deserialize");

        let mut find = self
            .collection
            .find(mongodb::bson::doc! { "discord_id": id }, None)
            .await?;

        let mut vec = Vec::new();

        while let Some(record) = find.next().await {
            vec.push(record?.minecraft_username);
        }

        Ok(vec)
    }

    async fn add_username(
        &self,
        id: serenity::UserId,
        username: String,
    ) -> Self::Result<Result<(), ErrorAlreadyClaimed>> {
        if let Some(user_id) = self.player_from_minecraft(&username).await? {
            Ok(Err(ErrorAlreadyClaimed::new(user_id, None, username)))
        } else {
            let record = Record {
                minecraft_username: username,
                discord_id: id,
            };

            self.collection.insert_one(record, None).await?;

            Ok(Ok(()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Players;
    type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

    mod consts {
        use poise::serenity_prelude as serenity;

        pub const USER_ID_FOO: serenity::UserId = serenity::UserId::new(1);
        pub const MINECRAFT_USERNAME_FOO: &str = "foo";

        pub const USER_ID_BAR: serenity::UserId = serenity::UserId::new(2);
        pub const MINECRAFT_USERNAME_BAR: &str = "bar";

        //pub const USER_ID_BAZ: serenity::UserId = serenity::UserId::new(3);
        pub const MINECRAFT_USERNAME_BAZ: &str = "baz";
    }

    macro_rules! test_backends {
        {$($backend:ty as $mod_name:ident $block:block)+} => {
            $(
            paste::paste! {
                mod [<$mod_name _ backend>] {
                    use super::consts::*;
                    use super::Result;
                    type Players = super::Players<super::$backend>;

                    #[tracing_test::traced_test]
                    #[tokio::test]
                    async fn add() -> Result {
                        let (players, _db) = $block;

                        players
                            .add_username(USER_ID_FOO, MINECRAFT_USERNAME_FOO.to_owned())
                            .await?
                            .expect("map is empty");

                        players
                            .add_username(USER_ID_BAR, MINECRAFT_USERNAME_BAR.to_owned())
                            .await?
                            .expect("not already added");

                        players
                            .add_username(USER_ID_FOO, MINECRAFT_USERNAME_BAZ.to_owned())
                            .await?
                            .expect("a user can claim multiple minecraft usernames");

                        Ok(())
                    }

                    #[tracing_test::traced_test]
                    #[tokio::test]
                    async fn already_claimed() -> Result {
                        let (players, _db) = $block;

                        players
                            .add_username(USER_ID_FOO, MINECRAFT_USERNAME_FOO.to_owned())
                            .await?
                            .expect("map is empty");

                        players
                            .add_username(USER_ID_BAR, MINECRAFT_USERNAME_FOO.to_owned())
                            .await?
                            .expect_err("a username cannot be claimed twice");

                        Ok(())
                    }

                    #[tracing_test::traced_test]
                    #[tokio::test]
                    async fn get_minecraft_username() -> Result {
                        let (players, _db) = $block;

                        players
                            .add_username(USER_ID_FOO, MINECRAFT_USERNAME_FOO.to_owned())
                            .await?
                            .expect("map is empty");

                        players
                            .add_username(USER_ID_BAR, MINECRAFT_USERNAME_BAR.to_owned())
                            .await?
                            .expect("not already added");

                        players
                            .add_username(USER_ID_FOO, MINECRAFT_USERNAME_BAZ.to_owned())
                            .await?
                            .expect("a user can claim multiple minecraft usernames");

                        assert!(players
                            .player_from_discord(USER_ID_FOO)
                            .await?
                            .contains(&MINECRAFT_USERNAME_FOO.to_string()));

                        assert!(players
                            .player_from_discord(USER_ID_FOO)
                            .await?
                            .contains(&MINECRAFT_USERNAME_BAZ.to_string()));

                        assert!(!players
                            .player_from_discord(USER_ID_BAR)
                            .await?
                            .contains(&MINECRAFT_USERNAME_FOO.to_string()));

                        Ok(())
                    }

                    #[tracing_test::traced_test]
                    #[tokio::test]
                    async fn get_discord_id() -> Result {
                        let (players, _db) = $block;

                        players
                            .add_username(USER_ID_FOO, MINECRAFT_USERNAME_FOO.to_owned())
                            .await?
                            .expect("map is empty");

                        players
                            .add_username(USER_ID_BAR, MINECRAFT_USERNAME_BAR.to_owned())
                            .await?
                            .expect("not already added");

                        players
                            .add_username(USER_ID_FOO, MINECRAFT_USERNAME_BAZ.to_owned())
                            .await?
                            .expect("a user can claim multiple minecraft usernames");

                        assert_eq!(
                            players
                                .player_from_minecraft(MINECRAFT_USERNAME_FOO)
                                .await?,
                            Some(USER_ID_FOO)
                        );

                        assert_eq!(
                            players
                                .player_from_minecraft(MINECRAFT_USERNAME_BAR)
                                .await?,
                            Some(USER_ID_BAR)
                        );

                        assert_eq!(
                            players
                                .player_from_minecraft(MINECRAFT_USERNAME_BAZ)
                                .await?,
                            Some(USER_ID_FOO)
                        );

                        Ok(())
                    }
                }
            }
            )+
        };
    }

    test_backends! {
        super::HashMap as hash_map {
            (Players::new(), ())
        }

        super::MongoDb as mongodb {
            let mongodb = temp_mongo::TempMongo::new()
            .await
            .expect("setting up db should not fail");
            let client = mongodb.client();
            let collection = client.database("slimebot_test").collection("minecraft");
            (Players::new(collection), mongodb)
        }
    }
}
