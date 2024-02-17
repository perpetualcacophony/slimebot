use std::sync::Arc;

use chrono::Duration;
use mongodb::{bson::doc, Collection, Database};
use poise::serenity_prelude::{User, UserId};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tracing::{instrument, warn};

type ArcMutex<T> = Arc<Mutex<T>>;

#[derive(Debug, Clone)]
pub struct Sprint {
    data: ArcMutex<SprintData>
}

impl Sprint {
    pub fn new() -> Self {
        let data = Arc::new(Mutex::new(
            SprintData::new()
        ));

        Self {
            data
        }
    }

    #[instrument(level = "trace", skip_all)]
    pub async fn add_member(&self, member: User) {
        let mut data = self.data.lock().await;
        let members = &mut data.members;

        if members.contains(&member) {
            warn!(?member.id, "member already in list, ignoring")
        } else {
            members.push(member)
        }
    }

    pub async fn members(&self) -> Vec<User> {
        let data = self.data.lock().await;
        data.members.clone()
    }
}

#[derive(Debug, Clone)]
struct SprintData {
    duration: Option<Duration>,
    members: Vec<User>,
}

impl SprintData {
    fn new() -> Self {
        Self {
            duration: None,
            members: Vec::new(),
        }
    }

    fn setup(&mut self, minutes: i64) {
        self.duration = Some(Duration::minutes(minutes))
    }

    fn start() {

    }

    fn finish(&self) {

    }
}

#[derive(Default, Serialize, Deserialize)]
struct SprintMember {
    id: UserId,
    #[serde(flatten)]
    stats: SprintStats,
}

impl SprintMember {
    fn new(id: UserId) -> Self {
        Self {
            id,
            stats: SprintStats::default()
        }
    }
}

#[derive(Default, Serialize, Deserialize)]
#[serde(default)] 
struct SprintStats {
    words: u32,
    most_words: u32,
    points: u32,
    wins: u32
}

trait SprintMemberData {
    fn sprint_members_collection(&self) -> Collection<SprintMember>;

    async fn get_sprint_member(&self, id: UserId) -> Option<SprintMember>;
    async fn insert_sprint_member(&self, member: SprintMember);
}

impl SprintMemberData for Database {
    fn sprint_members_collection(&self) -> Collection<SprintMember> {
        self.collection::<SprintMember>("sprint_members")
    }

    async fn get_sprint_member(&self, id: UserId) -> Option<SprintMember> {
        self.sprint_members_collection()
            .find_one(doc! { "id": id.to_string() }, None)
            .await
            .expect("db connection should work")
    }

    async fn insert_sprint_member(&self, member: SprintMember) {
        self.sprint_members_collection()
            .insert_one(member, None)
            .await
            .unwrap();
    }
}
