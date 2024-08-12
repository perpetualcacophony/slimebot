use poise::serenity_prelude as serenity;

mod url;
use url::ComicUrl as Url;

mod error;
pub use error::ParseComicError as ParseError;

type Result<T, E = ParseError> = std::result::Result<T, E>;

#[derive(Debug, Clone)]
pub struct ComicPage {
    title: String,
    url: Url,
    images: Vec<reqwest::Url>,
    date: String,
}

impl ComicPage {
    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn date(&self) -> &str {
        &self.date
    }

    pub fn url(&self) -> &Url {
        &self.url
    }

    pub fn slug(&self) -> &str {
        self.url().slug()
    }

    pub fn images(&self) -> impl Iterator<Item = &reqwest::Url> {
        self.images.iter()
    }

    pub async fn attachments(
        &self,
        http: &serenity::Http,
    ) -> serenity::Result<impl Iterator<Item = serenity::CreateAttachment>> {
        use serenity::{
            futures::stream::{self, StreamExt, TryStreamExt},
            CreateAttachment,
        };

        Ok(stream::iter(self.images())
            .then(|url| CreateAttachment::url(http, url.as_str()))
            .try_collect::<Vec<_>>()
            .await?
            .into_iter())
    }

    pub fn builder(&self) -> super::response::ResponseBuilder<'_> {
        super::response::ResponseBuilder::new(self)
    }
}

impl ComicPage {
    pub async fn random(client: &reqwest::Client) -> Result<Self> {
        let url = Url::random(client).await?;
        Self::from_slug(client, url.slug()).await
    }

    pub async fn from_slug(client: &reqwest::Client, slug: &str) -> Result<Self> {
        let url = Url::new(slug.to_owned());

        let text = client
            .get(url.to_string())
            .send()
            .await?
            .text()
            .await
            .expect("page should have text");

        let html = scraper::Html::parse_document(&text);

        let title = {
            let selector = "h1.entry-title";

            html.select(&scraper::Selector::parse(selector).expect("should be valid selector"))
                .next()
                .expect("page should have at least 1 comic title")
                .text()
                .next()
                .expect("should have text")
        }
        .to_owned();

        let date = {
            let selector = ".entry-meta>.posted-on>a";

            html.select(&scraper::Selector::parse(selector).expect("should be valid selector"))
                .next()
                .expect("page should have at least 1 comic date")
                .text()
                .next()
                .expect("should have text")
        }
        .to_owned();

        let images = {
            let selector = "#spliced-comic p img";

            html.select(&scraper::Selector::parse(selector).expect("should be valid selector"))
                .map(|img| img.attr("src").expect("img element should have src attr"))
                .map(|src| reqwest::Url::parse(src).expect("img src should be valid url"))
        }
        .collect();

        Ok(Self {
            title,
            url,
            images,
            date,
        })
    }

    pub async fn from_homepage(client: &reqwest::Client) -> Result<Self> {
        let homepage = reqwest::Url::parse(&format!("https://{host}", host = Url::HOST))
            .expect("url should be well-formed");

        let text = client
            .get(homepage.as_str())
            .send()
            .await?
            .text()
            .await
            .map_err(|_| error::NoTextError::new(homepage))?;

        let html = scraper::Html::parse_document(&text);

        let title = {
            let selector = ".entry-title";

            html.select(&scraper::Selector::parse(selector).expect("should be valid selector"))
                .next()
                .expect("page should have at least 1 comic title")
                .text()
                .next()
                .expect("should have text")
        }
        .to_owned();

        let (date, url) = {
            let selector = ".entry-meta>.posted-on>a";

            let element = html
                .select(&scraper::Selector::parse(selector).expect("should be valid selector"))
                .next()
                .expect("page should have at least 1 comic date");

            (
                element.text().next().expect("should have text").to_owned(),
                Url::parse(
                    element
                        .attr("href")
                        .expect("a element should have href attr"),
                )
                .expect("href should be valid comic url"),
            )
        }
        .to_owned();

        let images = {
            let selector = "#spliced-comic p img";

            html.select(&scraper::Selector::parse(selector).expect("should be valid selector"))
                .map(|img| img.attr("src").expect("img element should have src attr"))
                .map(|src| reqwest::Url::parse(src).expect("img src should be valid url"))
        }
        .collect();

        Ok(Self {
            title,
            url,
            images,
            date,
        })
    }
}
