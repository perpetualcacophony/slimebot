use poise::{serenity_prelude::CreateAttachment, CreateReply};
use serde::Deserialize;
use tracing::instrument;

use crate::{
    commands::LogCommands,
    utils::{
        poise::{CommandResult, ContextExt},
        Context,
    },
};

#[instrument(skip_all)]
#[poise::command(
    slash_command,
    prefix_command,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
pub async fn borzoi(ctx: Context<'_>) -> CommandResult {
    ctx.log_command().await;

    #[derive(Deserialize)]
    struct DogApiResponse {
        message: String,
    }

    let response = reqwest::get("https://dog.ceo/api/breed/borzoi/images/random").await?;

    if response.status().is_server_error() {
        ctx.reply_ext("sorry, dog api is down!").await?;

        //return Err(errors::CommandError::Internal(errors::InternalError::Api(
        //    ApiError::DogCeo(response.status().as_u16()),
        //)));
    }

    let image_url = response.json::<DogApiResponse>().await?.message;

    let attachment = CreateAttachment::url(&ctx, &image_url).await?;

    let reply = ctx.reply_builder(
        CreateReply::default()
            .content("borzoi courtesy of [dog.ceo](<https://dog.ceo/dog-api/>)")
            .attachment(attachment)
            .reply(true),
    );

    ctx.send_ext(reply).await?;

    Ok(())
}
