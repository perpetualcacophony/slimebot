use poise::serenity_prelude::{
    futures::future::join_all, CacheHttp, ChannelId, Color, Context, CreateEmbed, Reaction,
    ReactionType, UserId,
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
            .messages(ctx.http(), |get| {
                get.around(add_reaction.message_id).limit(5)
            })
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
        let footer_icon = UserId(ctx.http().http().application_id().unwrap())
            .to_user(ctx.http())
            .await
            .unwrap()
            .face();
        let member = add_reaction
            .member
            .unwrap()
            .guild_id
            .unwrap()
            .member(ctx.http(), add_reaction.user_id.unwrap())
            .await
            .unwrap();

        let mut embed = CreateEmbed::default();

        embed
            .title("bug report!")
            .author(|author| author.icon_url(member.face()).name(member.display_name()))
            .description(
                "react to a message with 🐞 to generate one of these reports!

            report context:",
            )
            .thumbnail("https://files.catbox.moe/0v4p11.png")
            .color(Color::from_rgb(221, 46, 68))
            .fields(messages)
            .footer(|footer| footer.icon_url(footer_icon).text("slimebot"))
            .timestamp(add_reaction.message_id.created_at());

        channel
            .send_message(ctx.http(), |msg| msg.set_embed(embed.clone()))
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
