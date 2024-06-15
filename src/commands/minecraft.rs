use crate::{commands::LogCommands, utils::poise::ContextExt};

use crate::utils::{poise::CommandResult, Context};
use poise::serenity_prelude::{self as serenity};
use tracing::{debug, instrument};

#[instrument(skip_all)]
#[poise::command(
    slash_command,
    prefix_command,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL",
    subcommands("claim")
)]
pub async fn minecraft(ctx: Context<'_>, server: Option<String>) -> crate::Result<()> {
    ctx.log_command().await;

    let result: CommandResult = try {
        let data = ctx.data().minecraft();

        let address = server.unwrap_or("162.218.211.126".to_owned());
        let response = api::Response::get(&address).await.map_err(Error::Api)?;

        match response {
            api::Response::Online(response) => {
                let mut players_fields = Vec::with_capacity(response.players.online);
                for player in &response.players {
                    let description = if let Some(serenity_id) = data
                        .players()
                        .player_from_minecraft(player.name())
                        .await
                        .expect("infallible")
                    {
                        let user = serenity_id.to_user(ctx).await?;

                        if let Some(guild_id) = ctx.guild_id() {
                            user.nick_in(ctx, guild_id).await.unwrap_or(user.name)
                        } else {
                            user.name
                        }
                    } else {
                        "".to_owned()
                    };

                    players_fields.push((player.name(), description, false))
                }

                let mut embed = serenity::CreateEmbed::new()
                    .title(format!(
                        "{host} ({version})",
                        host = response.host(),
                        version = response.version.name
                    ))
                    .description(format!(
                        "{motd}\n\nplayers online: {count}",
                        count = response.players.online,
                        motd = response.motd.clean()
                    ))
                    .fields(players_fields);

                if let Some(url) = response.icon_url().await.map_err(Error::Api)? {
                    debug!(%url);
                    embed = embed.thumbnail(url.to_string());
                }

                ctx.send_ext(poise::CreateReply::default().reply(true).embed(embed))
                    .await?;
            }
            api::Response::Offline(response) => {
                ctx.say_ext(format!(
                    "{host} is offline!",
                    host = response.host().map(ToString::to_string).unwrap_or(address)
                ))
                .await?;
            }
        }
    };

    result?;

    Ok(())
}

#[instrument(skip_all)]
#[poise::command(
    slash_command,
    prefix_command,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
pub async fn claim(ctx: Context<'_>, username: String) -> crate::Result<()> {
    let result: CommandResult = try {
        let data = ctx.data().minecraft();
        data.players()
            .add_username(ctx.author().id, username.clone())
            .await
            .expect("error is infallible")
            .map_err(Error::AlreadyClaimed)?;

        ctx.reply_ext(format!(
            "claimed minecraft account {name}!",
            name = username
        ))
        .await?;
    };

    result?;

    Ok(())
}

pub mod api {
    struct Request;

    impl Request {
        fn host() -> reqwest::Url {
            #[allow(clippy::unwrap_used)]
            reqwest::Url::parse("https://api.mcstatus.io").unwrap()
        }

        fn path() -> &'static str {
            "/v2/status/java/"
        }

        fn base_url() -> reqwest::Url {
            #[allow(clippy::unwrap_used)]
            Self::host().join(Self::path()).unwrap()
        }

        fn url(host: url::Host) -> Result<reqwest::Url, url::ParseError> {
            Self::base_url().join(&host.to_string())
        }

        async fn get_response(host: url::Host) -> Result<Response> {
            let client = reqwest::Client::new();
            let request = client
                .get(Self::url(host)?)
                .query(&[("query", false)])
                .build()?;
            let response = client.execute(request).await?;
            Ok(response
                .json()
                .await
                .expect("deserializing to Response should not fail"))
        }
    }

    #[derive(serde::Deserialize)]
    pub struct ResponseOffline {
        pub ip_address: Option<std::net::IpAddr>,
        pub host: Option<url::Host>,
    }

    impl ResponseOffline {
        pub fn host(&self) -> Option<&url::Host> {
            self.host.as_ref()
        }
    }

    #[derive(serde::Deserialize)]
    #[serde(untagged)]
    pub enum Response {
        Online(ResponseOnline),
        Offline(ResponseOffline),
    }

    impl Response {
        pub async fn from_host(address: url::Host) -> Result<Self> {
            Request::get_response(address).await
        }

        pub async fn get(address: &str) -> Result<Self> {
            let host = url::Host::parse(address)?;
            Self::from_host(host).await
        }
    }

    use crate::errors::TracingError;

    #[derive(Debug, thiserror::Error, TracingError)]
    #[span]
    pub enum Error {
        #[error(transparent)]
        #[event(level = WARN)]
        ParseUrl(#[from] url::ParseError),

        #[error(transparent)]
        #[event(level = ERROR)]
        Server(reqwest::Error),

        #[error(transparent)]
        #[event(level = ERROR)]
        Unknown(reqwest::Error),
    }

    impl From<reqwest::Error> for Error {
        fn from(error: reqwest::Error) -> Self {
            if error
                .status()
                .as_ref()
                .is_some_and(reqwest::StatusCode::is_server_error)
            {
                Self::Server(error)
            } else {
                Self::Unknown(error)
            }
        }
    }

    type Result<T, E = Error> = std::result::Result<T, E>;

    #[derive(serde::Deserialize)]
    pub struct Version {
        #[serde(rename = "name_clean")]
        pub name: String,
    }

    mod players {
        #[derive(serde::Deserialize)]
        pub struct Players {
            pub online: usize,
            pub max: usize,
            pub list: Vec<super::Player>,
        }

        impl Players {
            pub fn iter(&self) -> Iter<'_> {
                self.into_iter()
            }
        }

        type IterInner<'p> = std::slice::Iter<'p, super::Player>;
        pub struct Iter<'p>(IterInner<'p>);

        impl<'p> From<IterInner<'p>> for Iter<'p> {
            fn from(value: IterInner<'p>) -> Self {
                Self(value)
            }
        }

        impl<'p> Iterator for Iter<'p> {
            type Item = <IterInner<'p> as Iterator>::Item;

            fn next(&mut self) -> Option<Self::Item> {
                self.0.next()
            }
        }

        impl<'p> IntoIterator for &'p Players {
            type IntoIter = Iter<'p>;
            type Item = <Iter<'p> as Iterator>::Item;

            fn into_iter(self) -> Self::IntoIter {
                self.list.iter().into()
            }
        }
    }
    use base64::Engine;
    pub use players::Players;
    use slimebot_macros::TracingError;
    use tracing::debug;

    #[derive(serde::Deserialize)]
    pub struct Player {
        #[serde(rename = "name_clean")]
        pub name: String,
    }

    impl Player {
        pub fn name(&self) -> &str {
            &self.name
        }
    }

    #[derive(serde::Deserialize)]
    pub struct ResponseOnline {
        host: String,

        pub version: Version,

        pub players: Players,

        pub motd: Motd,

        icon: Option<String>,
    }

    impl ResponseOnline {
        pub fn host(&self) -> &str {
            &self.host
        }

        pub async fn icon_url(&self) -> Result<Option<reqwest::Url>> {
            if let Some(ref icon) = self.icon {
                let base64 = icon
                    .strip_prefix("data:image/png;base64,")
                    .expect("should start with png header")
                    .to_owned();

                let image_data = base64::prelude::BASE64_STANDARD
                    .decode(base64)
                    .expect("field should be valid base64");

                let file_to_upload =
                    reqwest::multipart::Part::bytes(image_data).file_name("image.png");

                let form = reqwest::multipart::Form::new()
                    .text("reqtype", "fileupload")
                    .text("time", "72h")
                    .part("fileToUpload", file_to_upload);

                let response = reqwest::Client::new()
                    .post("https://litterbox.catbox.moe/resources/internals/api.php ")
                    .multipart(form)
                    .send()
                    .await?;

                debug!(code = %response.status());

                Ok(Some(
                    reqwest::Url::parse(&response.text().await.expect("response should have text"))
                        .expect("response should be a url"),
                ))
            } else {
                Ok(None)
            }
        }
    }

    #[derive(serde::Deserialize)]
    pub struct Motd {
        clean: String,
    }

    impl Motd {
        pub fn clean(&self) -> &str {
            &self.clean
        }
    }
}

mod players {
    use poise::serenity_prelude as serenity;

    use super::ErrorAlreadyClaimed;

    pub trait Backend {
        type Error;
        type Result<T> = std::result::Result<T, Self::Error>;

        // read
        async fn player_from_minecraft(
            &self,
            username: &str,
        ) -> Self::Result<Option<serenity::UserId>>;
        async fn player_from_discord(&self, id: serenity::UserId) -> Self::Result<Vec<String>>;

        // update
        async fn add_username(
            &self,
            id: serenity::UserId,
            username: String,
        ) -> Self::Result<Result<(), ErrorAlreadyClaimed>>;
    }

    #[derive(Debug, Default)]
    pub struct Players<Backend> {
        backend: std::sync::Arc<Backend>,
    }

    impl<B> Clone for Players<B> {
        fn clone(&self) -> Self {
            Self {
                backend: self.backend.clone(),
            }
        }
    }

    impl<B: Backend> Players<B> {
        pub async fn player_from_minecraft(
            &self,
            username: &str,
        ) -> B::Result<Option<serenity::UserId>> {
            self.backend.player_from_minecraft(username).await
        }

        pub async fn player_from_discord(&self, id: serenity::UserId) -> B::Result<Vec<String>> {
            self.backend.player_from_discord(id).await
        }

        pub async fn add_username(
            &self,
            id: serenity::UserId,
            username: String,
        ) -> B::Result<Result<(), ErrorAlreadyClaimed>> {
            self.backend.add_username(id, username).await
        }
    }

    pub type HashMap = tokio::sync::RwLock<std::collections::HashMap<String, serenity::UserId>>;

    pub type PlayersHashMap = Players<HashMap>;

    impl PlayersHashMap {
        pub fn new() -> Self {
            Self::default()
        }
    }

    impl BackendInfallible
        for tokio::sync::RwLock<std::collections::HashMap<String, serenity::UserId>>
    {
        async fn player_from_minecraft(&self, username: &str) -> Option<serenity::UserId> {
            let guard = self.read().await;
            guard.get(username).copied()
        }

        async fn player_from_discord(&self, id: serenity::UserId) -> Vec<String> {
            let guard = self.read().await;
            guard
                .iter()
                .filter_map(|(name, val_id)| (*val_id == id).then_some(name.clone()))
                .collect()
        }

        async fn add_username(
            &self,
            id: serenity::UserId,
            username: String,
        ) -> Result<(), ErrorAlreadyClaimed> {
            let mut guard = self.write().await;
            if guard.insert(username.clone(), id).is_some() {
                Err(ErrorAlreadyClaimed::new(id, None, username))
            } else {
                Ok(())
            }
        }
    }

    trait BackendInfallible {
        async fn player_from_minecraft(&self, username: &str) -> Option<serenity::UserId>;
        async fn player_from_discord(&self, id: serenity::UserId) -> Vec<String>;
        async fn add_username(
            &self,
            id: serenity::UserId,
            username: String,
        ) -> Result<(), ErrorAlreadyClaimed>;
    }
    impl<B: BackendInfallible> Backend for B {
        type Error = std::convert::Infallible;

        async fn player_from_minecraft(
            &self,
            username: &str,
        ) -> Self::Result<Option<serenity::UserId>> {
            Ok(BackendInfallible::player_from_minecraft(self, username).await)
        }

        async fn player_from_discord(&self, id: serenity::UserId) -> Self::Result<Vec<String>> {
            Ok(BackendInfallible::player_from_discord(self, id).await)
        }

        async fn add_username(
            &self,
            id: serenity::UserId,
            username: String,
        ) -> Self::Result<Result<(), ErrorAlreadyClaimed>> {
            Ok(BackendInfallible::add_username(self, id, username).await)
        }
    }
}
pub use players::Players;

#[derive(Debug)]
pub struct Data<B = players::HashMap> {
    players: Players<B>,
}

impl<B> Data<B> {
    fn players(&self) -> Players<B> {
        self.players.clone()
    }
}

impl Data<players::HashMap> {
    pub fn new() -> Self {
        Self {
            players: Players::new(),
        }
    }
}

impl<B> Clone for Data<B> {
    fn clone(&self) -> Self {
        Self {
            players: self.players.clone(),
        }
    }
}

use crate::errors::TracingError;
#[derive(Debug, Clone, thiserror::Error, slimebot_macros::TracingError)]
#[event(level = WARN)]
pub struct ErrorAlreadyClaimed {
    #[field(print = Display)]
    user_id: serenity::UserId,
    user_name: Option<String>,
    minecraft_username: String,
}

impl ErrorAlreadyClaimed {
    fn new(
        user_id: serenity::UserId,
        user_name: Option<String>,
        minecraft_username: String,
    ) -> Self {
        Self {
            user_id,
            user_name,
            minecraft_username,
        }
    }

    fn from_serenity(user: &serenity::User, minecraft_username: String) -> Self {
        Self::new(user.id, Some(user.name.clone()), minecraft_username)
    }

    fn set_user_name(&mut self, new: String) {
        self.user_name = Some(new)
    }

    pub async fn update_user_name(
        &mut self,
        cache_http: impl serenity::CacheHttp,
    ) -> serenity::Result<()> {
        let user = self.user_id.to_user(cache_http).await?;
        self.set_user_name(user.name);
        Ok(())
    }

    pub async fn update_user_nick(
        &mut self,
        cache_http: impl serenity::CacheHttp,
        guild_id: Option<serenity::GuildId>,
    ) -> serenity::Result<()> {
        let user = self.user_id.to_user(&cache_http).await?;
        let nick = if let Some(guild_id) = guild_id {
            user.nick_in(&cache_http, guild_id)
                .await
                .unwrap_or(user.name)
        } else {
            user.name
        };
        self.set_user_name(nick);
        Ok(())
    }
}

impl std::fmt::Display for ErrorAlreadyClaimed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let already_claimed_by = if let Some(ref name) = self.user_name {
            format!("{name} ({id})", id = self.user_id)
        } else {
            format!("user {id}", id = self.user_id)
        };

        write!(
            f,
            "minecraft user {username} already claimed by {already_claimed_by}",
            username = self.minecraft_username
        )
    }
}

#[derive(Debug, thiserror::Error, slimebot_macros::TracingError)]
pub enum Error {
    #[error(transparent)]
    AlreadyClaimed(#[from] ErrorAlreadyClaimed),
    #[error(transparent)]
    Api(#[from] api::Error),
}
