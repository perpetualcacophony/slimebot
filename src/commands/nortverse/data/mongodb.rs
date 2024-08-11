use poise::serenity_prelude as serenity;

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
                    "id": to_bson(&id).unwrap()
                },
                None,
            )
            .await
            .map(|_| ())
    }
}
