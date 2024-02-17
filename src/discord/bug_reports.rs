use poise::serenity_prelude::{
    futures::future::join_all, ChannelId, Color, CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter, CreateMessage, GetMessages, Http, Reaction, ReactionType, UserId
};

#[allow(unused_imports)]
use tracing::{debug, info, trace};

pub async fn bug_reports(http: &Http, add_reaction: Reaction, channel: &ChannelId) {
    let ladybug_reaction = ReactionType::Unicode("🐞".to_string());

    let ladybugs = add_reaction
        .message(http)
        .await
        .expect("reaction should have a message")
        .reactions
        .iter()
        .find(|r| r.reaction_type == ladybug_reaction)
        .map_or_else(|| 0, |r| r.count);

    let add_after = add_reaction.clone();

    if add_reaction.emoji == ladybug_reaction && ladybugs == 1 {
        let messages = add_reaction
            .channel_id
            .messages(
                http,
                GetMessages::new().around(add_reaction.message_id).limit(5),
            )
            .await
            .expect("channel should have messages");

        let messages = messages
            .into_iter()
            .rev()
            .enumerate()
            .map(|(n, m)| async move {
                let content = if m.content.is_empty() {
                    "*empty message*"
                } else {
                    &m.content
                };

                let name = m.author_nick(http).await.unwrap_or(m.author.name);

                if n == 2 {
                    (
                        format!(
                            "{name} << bug occurred here {}",
                            add_reaction
                                .message_id
                                .link(add_reaction.channel_id, add_reaction.guild_id)
                        ),
                        format!("**{}**", content),
                        false,
                    )
                } else {
                    (name, content.to_string(), false)
                }
            });

        let messages = join_all(messages).await;
        let footer_icon = UserId::new(http.application_id().expect("bot app should have id").get())
            .to_user(http)
            .await
            .expect("user id should match a user")
            .face();
        let member = add_reaction
            .member
            .expect("reaction in guild should have member")
            .guild_id
            .member(http, add_reaction.user_id.expect("reaction should have user id"))
            .await
            .expect("user id should match member");

        let mut embed = CreateEmbed::default();

        embed = embed
            .title("bug report!")
            .author(CreateEmbedAuthor::new(member.display_name()).icon_url(member.face()))
            .description(
                "react to a message with 🐞 to generate one of these reports!

            report context:",
            )
            .thumbnail("https://files.catbox.moe/0v4p11.png")
            .color(Color::from_rgb(221, 46, 68))
            .fields(messages)
            .footer(CreateEmbedFooter::new("slimebot").icon_url(footer_icon))
            .timestamp(add_reaction.message_id.created_at());

        channel
            .send_message(http, CreateMessage::new().embed(embed.clone()))
            .await
            .expect("sending message should not fail");

        info!(
            "@{} reported a bug: {} (#{})",
            add_after.user(http).await.expect("reaction should have author").name,
            add_after.message_id,
            add_after
                .channel(http)
                .await
                .expect("reaction should have channel")
                .guild()
                .expect("channel should be in guild")
                .name(),
        );
    }
}
