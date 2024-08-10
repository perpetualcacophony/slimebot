use poise::serenity_prelude as serenity;

#[derive(Default, Debug)]
pub struct IssueBuilder {
    title: String,
    body: Option<String>,
    guild: Option<serenity::GuildId>,
    author: serenity::UserId,
    messages: Vec<serenity::MessageId>,
}

impl IssueBuilder {
    pub fn new(title: String, author: serenity::UserId) -> Self {
        Self {
            title,
            author,
            ..Default::default()
        }
    }

    pub fn body(mut self, body: String) -> Self {
        self.body = Some(body);
        self
    }

    pub fn guild(mut self, guild: serenity::GuildId) -> Self {
        self.guild = Some(guild);
        self
    }

    pub fn messages(mut self, messages: impl IntoIterator<Item = serenity::MessageId>) -> Self {
        self.messages = Vec::from_iter(messages);
        self
    }
}
