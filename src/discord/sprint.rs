use chrono::Duration;
use mongodb::{bson::doc, Collection, Database};
use poise::serenity_prelude::{Member, UserId};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast::Receiver;

struct Sprint {
    duration: Duration,
    members: Vec<Member>,
    words_receiver: Receiver<u32>,
}

impl Sprint {
    fn new(rx: Receiver<u32>, minutes: i64) -> Self {
        let duration = Duration::minutes(minutes);
        
        Self {
            duration,
            members: Vec::new(),
            words_receiver: rx,
        }
    }

    fn add_member(&mut self, member: Member) {
        self.members.push(member);
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
