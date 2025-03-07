use poise::serenity_prelude as serenity;

#[derive(Debug, Clone)]
pub struct MongoDb {
    latest: mongodb::Collection<SlugRecord>,
    subscribers: mongodb::Collection<SubscriberRecord>,
}

impl MongoDb {
    const LATEST_COLLECTION_NAME: &str = "nortverse_latest";
    const SUBSCRIBERS_COLLECTION_NAME: &str = "nortverse_subscribers";

    pub fn from_database(db: &mongodb::Database) -> Self {
        Self {
            latest: db.collection(Self::LATEST_COLLECTION_NAME),
            subscribers: db.collection(Self::SUBSCRIBERS_COLLECTION_NAME),
        }
    }
}

impl<'a> From<&'a mongodb::Database> for MongoDb {
    fn from(value: &'a mongodb::Database) -> Self {
        Self::from_database(value)
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct SlugRecord {
    slug: String,
    added: mongodb::bson::DateTime,
}

#[derive(Copy, Clone, Debug, serde::Serialize, serde::Deserialize)]
struct SubscriberRecord {
    id: serenity::UserId,
    added: mongodb::bson::DateTime,
}

impl super::NortverseDataAsync for MongoDb {
    type Error = mongodb::error::Error;

    async fn latest_slug(&self) -> Result<Option<impl AsRef<str>>, Self::Error> {
        use mongodb::{bson::doc, options::FindOneOptions};

        Ok(self
            .latest
            .find_one(
                None,
                FindOneOptions::builder()
                    .sort(doc! {
                        "added": -1
                    })
                    .build(),
            )
            .await?
            .map(|record| record.slug))
    }

    async fn set_latest(&mut self, slug: String) -> Result<(), Self::Error> {
        use mongodb::bson::DateTime;

        let record = SlugRecord {
            slug,
            added: DateTime::now(),
        };

        self.latest.insert_one(record, None).await.map(|_| ())
    }

    async fn subscribers(&self) -> Result<impl IntoIterator<Item = serenity::UserId>, Self::Error> {
        use serenity::futures::TryStreamExt;

        self.subscribers
            .find(None, None)
            .await?
            .map_ok(|record| record.id)
            .try_collect::<Vec<_>>()
            .await
    }

    async fn add_subscriber(&mut self, id: serenity::UserId) -> Result<(), Self::Error> {
        use mongodb::bson::DateTime;

        let record = SubscriberRecord {
            id,
            added: DateTime::now(),
        };

        self.subscribers.insert_one(record, None).await.map(|_| ())
    }

    async fn remove_subscriber(&mut self, id: serenity::UserId) -> Result<(), Self::Error> {
        use mongodb::bson::{doc, ser::to_bson};

        self.subscribers
            .delete_one(
                doc! {
                    "id": to_bson(&id).expect("discord id should be serializable")
                },
                None,
            )
            .await
            .map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use poise::serenity_prelude::UserId;
    use temp_mongo::TempMongo;

    use crate::data::NortverseDataAsync;

    use super::MongoDb;

    use pretty_assertions::{assert_eq, assert_ne};

    async fn temp_mongo() -> TempMongo {
        TempMongo::new()
            .await
            .expect("creating temp mongo should not fail")
    }

    fn get_db(mongo: &TempMongo) -> mongodb::Database {
        mongo.client().database("nortverse_data_test")
    }

    #[tokio::test]
    async fn set_latest() -> Result<(), mongodb::error::Error> {
        let mongo = temp_mongo().await;
        let mut data = MongoDb::from_database(&get_db(&mongo));

        data.set_latest("amber".to_owned()).await?;

        let data_slug = data.latest_slug().await?;
        let data_slug = data_slug.as_ref().map(AsRef::as_ref);

        assert_eq!(data_slug, Some("amber"));

        Ok(())
    }

    #[tokio::test]
    async fn update_latest() -> Result<(), mongodb::error::Error> {
        let mongo = temp_mongo().await;
        let mut data = MongoDb::from_database(&get_db(&mongo));

        {
            data.set_latest("amber".to_owned()).await?;

            let data_slug = data.latest_slug().await?;
            let data_slug = data_slug.as_ref().map(AsRef::as_ref);

            assert_eq!(data_slug, Some("amber"));
        }

        {
            data.set_latest("crimped".to_owned()).await?;

            let data_slug = data.latest_slug().await?;
            let data_slug = data_slug.as_ref().map(AsRef::as_ref);

            assert_eq!(data_slug, Some("crimped"));
            assert_ne!(data_slug, Some("amber"));
        }

        Ok(())
    }

    #[tokio::test]
    async fn add_subscriber() -> Result<(), mongodb::error::Error> {
        let mongo = temp_mongo().await;
        let mut data = MongoDb::from_database(&get_db(&mongo));

        data.add_subscriber(UserId::new(123)).await?;

        assert!(data.contains_subscriber(&UserId::new(123)).await?);
        assert!(!data.contains_subscriber(&UserId::new(456)).await?);

        Ok(())
    }

    #[tokio::test]
    async fn remove_subscriber() -> Result<(), mongodb::error::Error> {
        let mongo = temp_mongo().await;
        let mut data = MongoDb::from_database(&get_db(&mongo));

        data.add_subscriber(UserId::new(123)).await?;
        assert!(data.contains_subscriber(&UserId::new(123)).await?);

        data.remove_subscriber(UserId::new(123)).await?;
        assert!(!data.contains_subscriber(&UserId::new(123)).await?);
        assert!(!data.contains_subscriber(&UserId::new(456)).await?);

        data.add_subscriber(UserId::new(456)).await?;
        assert!(data.contains_subscriber(&UserId::new(456)).await?);

        data.remove_subscriber(UserId::new(456)).await?;
        assert!(!data.contains_subscriber(&UserId::new(456)).await?);

        Ok(())
    }

    #[tokio::test]
    async fn subscribers() -> Result<(), mongodb::error::Error> {
        let mongo = temp_mongo().await;
        let mut data = MongoDb::from_database(&get_db(&mongo));

        data.add_subscriber(UserId::new(123)).await?;
        data.add_subscriber(UserId::new(789)).await?;

        let subscribers = Vec::from_iter(data.subscribers().await?);

        assert_eq!(subscribers.len(), 2);
        assert!(subscribers.iter().any(|id| id == &UserId::new(123)));
        assert!(subscribers.iter().any(|id| id == &UserId::new(789)));
        assert!(!subscribers.iter().any(|id| id == &UserId::new(456)));

        drop(subscribers);

        data.remove_subscriber(UserId::new(123)).await?;

        let subscribers = Vec::from_iter(data.subscribers().await?);

        assert_eq!(subscribers.len(), 1);
        assert!(!subscribers.iter().any(|id| id == &UserId::new(123)));
        assert!(subscribers.iter().any(|id| id == &UserId::new(789)));

        Ok(())
    }
}
