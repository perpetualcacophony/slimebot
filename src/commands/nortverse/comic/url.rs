use std::{fmt::Display, str::FromStr};

#[derive(Debug, Clone)]
pub struct ComicUrl {
    slug: String,
}

impl ComicUrl {
    pub const HOST: &str = "nortverse.com";
    const COMICS_PATH: &str = "comic";
    const RANDOM_PATH: &str = "random";

    pub fn new(slug: String) -> Self {
        Self { slug }
    }

    pub fn slug(&self) -> &str {
        &self.slug
    }

    pub fn reqwest(&self) -> reqwest::Url {
        self.into()
    }

    pub fn parse(s: &str) -> Result<Self, ParseError> {
        s.parse()
    }

    pub async fn random(client: &reqwest::Client) -> reqwest::Result<Self> {
        let response = client
            .get(format!(
                "https://{host}/{path}?r={seed}",
                host = Self::HOST,
                path = Self::RANDOM_PATH,
                seed = rand::random::<u64>()
            ))
            .send()
            .await?;

        Ok(Self::try_from(response.url()).expect("should be a valid comic url"))
    }
}

impl Display for ComicUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "https://{host}/{path}/{slug}",
            host = Self::HOST,
            path = Self::COMICS_PATH,
            slug = self.slug
        )
    }
}

impl<'a> From<&'a ComicUrl> for reqwest::Url {
    fn from(value: &'a ComicUrl) -> Self {
        Self::parse(&value.to_string()).expect("should be valid url")
    }
}

impl<'a> TryFrom<&'a reqwest::Url> for ComicUrl {
    type Error = ParseError;

    fn try_from(value: &'a reqwest::Url) -> Result<Self, Self::Error> {
        let mut segments = value.path_segments().ok_or(ParseError::NotNortverse)?;

        let path = segments.next().ok_or(ParseError::NoPath)?;

        if path != Self::COMICS_PATH {
            return Err(ParseError::WrongPath);
        }

        let slug = segments.next().ok_or(ParseError::NoSlug)?;

        Ok(Self::new(slug.to_owned()))
    }
}

impl FromStr for ComicUrl {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(&reqwest::Url::parse(s)?)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("path doesn't match nortverse comics")]
    WrongPath,

    #[error("doesn't have a path")]
    NoPath,

    #[error("missing a comic slug")]
    NoSlug,

    #[error("not a nortverse url")]
    NotNortverse,

    #[error(transparent)]
    Url(#[from] url::ParseError),
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::ComicUrl;
    use pretty_assertions::assert_str_eq;

    #[test]
    fn from_str() {
        let url = ComicUrl::from_str("https://nortverse.com/comic/christmas-special-4/")
            .expect("parsing url should not fail");

        assert_str_eq!(
            url.to_string(),
            "https://nortverse.com/comic/christmas-special-4"
        )
    }

    #[test]
    fn slug() {
        let url = ComicUrl::from_str("https://nortverse.com/comic/christmas-special-4")
            .expect("parsing url should not fail");

        assert_str_eq!(url.slug(), "christmas-special-4")
    }
}
