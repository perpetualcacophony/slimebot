use poise::serenity_prelude::{
    futures::future::join_all, CacheHttp, ChannelId, Color, Context, CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter, CreateMessage, GetMessages, Reaction, ReactionType, UserId
};

#[allow(unused_imports)]
use tracing::debug;
use tracing::info;

pub async fn bug_reports(ctx: &Context, add_reaction: Reaction, channel: &ChannelId) {
    let ladybug_reaction = ReactionType::Unicode("🐞".to_string());

    let ladybugs = add_reaction
        .message(ctx.http())
        .await
        .unwrap()
        .reactions
        .iter()
        .find(|r| r.reaction_type == ladybug_reaction)
        .unwrap()
        .count;

    let add_after = add_reaction.clone();

    if add_reaction.emoji == ladybug_reaction && ladybugs == 1 {
        let messages = add_reaction
            .channel_id
            .messages(ctx.http(), GetMessages::new().around(add_reaction.message_id).limit(5))
            .await
            .unwrap();

        let http = ctx.http();

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
        let footer_icon = UserId::new(ctx.http().http().application_id().unwrap().get())
            .to_user(ctx.http())
            .await
            .unwrap()
            .face();
        let member = add_reaction
            .member
            .unwrap()
            .guild_id
            .member(ctx.http(), add_reaction.user_id.unwrap())
            .await
            .unwrap();

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
            .send_message(ctx.http(), CreateMessage::new().embed(embed.clone()))
            .await
            .unwrap();

        info!(
            "@{} reported a bug: {} (#{})",
            add_after.user(ctx.http()).await.unwrap().name,
            add_after.message_id,
            add_after
                .channel(ctx.http())
                .await
                .unwrap() // todo: handle the http request failing
                .guild()
                .unwrap() // this is ok - the report will not be outside a guild
                .name(),
        );
    }
}
