const PUZZLE_ACTIVE_HOURS: i64 = 24;

mod error;

mod as_emoji;
pub use as_emoji::AsEmoji;

use mongodb::error::Error as MongoDbError;

mod puzzle;
pub use puzzle::Puzzle;

type DbResult<T> = std::result::Result<T, MongoDbError>;

//mod words_list;
//pub use words_list::WordsList;

mod daily;
pub use daily::DailyWordles;

pub mod game;
pub use game::{Game, GameRecord};

mod data;
pub use data::WordleData;

/*
fn create_menu(daily_available: bool) -> CreateReply {
    let menu_text = if daily_available {
        "you have a daily wordle available!"
    } else {
        "you do not have a daily wordle available! play a random wordle?"
    };

    CreateReply::new()
        .content(menu_text)
        .button(
            CreateButton::new("daily")
                .label("daily")
                .emoji(ReactionType::Unicode("📅".to_owned()))
                .style(poise::serenity_prelude::ButtonStyle::Primary)
                .disabled(!daily_available),
        )
        .button(
            CreateButton::new("random")
                .label("random")
                .emoji(ReactionType::Unicode("🎲".to_owned()))
                .style(poise::serenity_prelude::ButtonStyle::Secondary),
        )
        .button(
            CreateButton::new("cancel")
                .label("cancel")
                .emoji(ReactionType::Unicode("🚫".to_owned()))
                .style(poise::serenity_prelude::ButtonStyle::Secondary),
        )
        .reply(true)
}
*/

/*async fn mode_select_menu(
    ctx: crate::Context<'_>,
    daily_wordles: DailyWordles,
    options: GameOptions,
) -> Result<(GameType, Message)> {
    let in_guild = ctx.in_guild();
    let playable = daily_wordles.playable_for(options.owner).await?;

    let next_daily = playable.next();

    let menu_builder = create_menu(next_daily.is_some());
    let menu = ctx.send(menu_builder).await?.into_message().await?;

    if let Some(interaction) = menu.await_component_interaction(ctx).await {
        let channel = if interaction.data.custom_id.as_str() == "daily" && in_guild {
            owner.create_dm_channel(ctx).await?.id
        } else {
            ctx.channel_id()
        };

        let (mode, menu) = match interaction.data.custom_id.as_str() {
            "daily" => {
                let message = if ctx.in_guild() {
                    interaction
                        .reply_ephemeral(
                            ctx,
                            "you can't play a daily wordle in a server - check your dms!",
                        )
                        .await?;

                    menu.delete(ctx).await?;

                    channel.say(ctx, "loading daily wordle...").await?
                } else {
                    interaction
                        .update_message(
                            ctx,
                            CreateInteractionResponseMessage::new()
                                .content("loading daily wordle...")
                                .components(Vec::new()),
                        )
                        .await?;

                    menu
                };

                (GameType::Daily, message)
            }
            "random" => {
                interaction
                    .update_message(
                        ctx,
                        CreateInteractionResponseMessage::new()
                            .content("loading random wordle...")
                            .components(Vec::new()),
                    )
                    .await?;

                (GameType::Random, menu)
            }
            "cancel" => {
                interaction
                    .update_message(
                        ctx,
                        CreateInteractionResponseMessage::new()
                            .content("canceled!")
                            .components(Vec::new()),
                    )
                    .await?;

                return Ok(());
            }
            _ => unreachable!(),
        };

        Ok((mode, menu))
    } else {
        panic!()
    }
}*/
