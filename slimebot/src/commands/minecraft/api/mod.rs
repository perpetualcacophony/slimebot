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
            .build()
            .expect("building request should not fail");

        let response = client
            .execute(request)
            .await
            .map_err(errors::ReqwestClientError::or_server::<errors::MinecraftApiError>)?;

        Ok(response
            .json()
            .await
            .expect("deserializing to Response should not fail"))
    }
}

#[derive(serde::Deserialize)]
pub struct ResponseOffline {
    //pub ip_address: Option<std::net::IpAddr>,
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

pub mod errors;
pub use errors::Error;

type Result<T, E = errors::Error> = std::result::Result<T, E>;

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
                .expect("icon field should be valid base64");

            let file_to_upload = reqwest::multipart::Part::bytes(image_data).file_name("image.png");

            let form = reqwest::multipart::Form::new()
                .text("reqtype", "fileupload")
                .text("time", "72h")
                .part("fileToUpload", file_to_upload);

            let response = reqwest::Client::new()
                .post("https://litterbox.catbox.moe/resources/internals/api.php ")
                .multipart(form)
                .send()
                .await
                .map_err(errors::ReqwestClientError::or_server::<errors::ImageHostError>)?;

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
