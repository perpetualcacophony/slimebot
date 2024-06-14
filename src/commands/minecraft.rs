use crate::{commands::LogCommands, utils::poise::ContextExt};

use crate::utils::{poise::CommandResult, Context};
use poise::serenity_prelude as serenity;
use tracing::{debug, instrument};

#[instrument(skip_all)]
#[poise::command(
    slash_command,
    prefix_command,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
pub async fn minecraft(ctx: Context<'_>, server: Option<String>) -> crate::Result<()> {
    ctx.log_command().await;

    let result: CommandResult = try {
        let address = server.unwrap_or("162.218.211.126".to_owned());
        let response = api::Response::get(&address).await?;

        match response {
            api::Response::Online(response) => {
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
                    .fields(
                        response
                            .players
                            .iter()
                            .map(|player| (player.name(), "", false)),
                    );

                if let Some(url) = response.icon_url().await? {
                    debug!(%url);
                    embed = embed.thumbnail(url.to_string());
                }

                ctx.send_ext(poise::CreateReply::default().reply(true).embed(embed))
                    .await?;
            }
            api::Response::Offline(response) => {
                todo!()
            }
        }
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
            Ok(response.json().await?)
        }
    }

    #[derive(serde::Deserialize)]
    pub struct ResponseOffline {
        //pub ip_address: IpAddr,
        //pub host: url::Host,
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
        pub host: String,

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

                let image_data = base64::prelude::BASE64_STANDARD.decode(base64).unwrap();

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

                Ok(Some(reqwest::Url::parse(&response.text().await?)?))
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
