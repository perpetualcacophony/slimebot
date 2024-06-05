use crate::{commands::LogCommands, utils::poise::ContextExt};

use crate::utils::{poise::CommandResult, Context};
use poise::{serenity_prelude::CreateEmbed, CreateReply};
use serde::Deserialize;
use tracing::{debug, instrument};

#[derive(Deserialize, Clone, Debug)]
struct ApiResponse {
    online: bool,
    version: Option<ApiResponseVersion>,
    players: Option<ApiResponsePlayers>,
}

impl ApiResponse {
    fn version(&self) -> &ApiResponseVersion {
        self.version
            .as_ref()
            .expect("online api response should have version")
    }

    fn players(&self) -> &ApiResponsePlayers {
        self.players
            .as_ref()
            .expect("online api response should have players")
    }
}

#[derive(Deserialize, Clone, Debug)]
struct ApiResponseVersion {
    #[serde(rename = "name_clean")]
    name_clean: String,
}

#[derive(Deserialize, Clone, Debug)]
struct ApiResponsePlayers {
    online: u8,
    #[serde(rename = "list")]
    list: Vec<ApiResponsePlayer>,
}

#[derive(Deserialize, Clone, Debug)]
struct ApiResponsePlayer {
    name_clean: String,
}

#[instrument(skip_all)]
#[poise::command(
    slash_command,
    prefix_command,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
pub async fn minecraft(ctx: Context<'_>, server: Option<String>) -> CommandResult {
    ctx.log_command().await;

    let address = server.unwrap_or("162.218.211.126".to_owned());
    let request_url = format!("https://api.mcstatus.io/v2/status/java/{address}");

    let response = reqwest::get(request_url)
        .await?
        .json::<ApiResponse>()
        .await?;

    debug!("{:#?}", response);

    let mut embed = CreateEmbed::default();
    embed = embed.title(address);

    if response.online {
        let players_online = response.players().online;
        embed = embed.description(format!("players online: {players_online}"));

        embed = embed.fields(
            response
                .players()
                .list
                .iter()
                .map(|p| (&p.name_clean, "", false)),
        );
    }

    ctx.send_ext(CreateReply::default().embed(embed)).await?;

    Ok(())
}
