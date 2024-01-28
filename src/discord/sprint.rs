use chrono::Duration;
use poise::serenity_prelude::Member;

struct Sprint {
    duration: Duration,
    members: Vec<Member>
}

impl Sprint {
    fn new(minutes: i64) -> Self {
        let duration = Duration::minutes(minutes);
        
        Self {
            duration,
            members: Vec::new(),
        }
    }

    fn add_member(&mut self, member: Member) {
        self.members.push(member);
    }
}

struct SprintMember {
    member: Member,
    stats: SprintStats,
}

struct SprintStats {
    words: u32,
    most_words: u32,
    points: u32,
    wins: u32
}