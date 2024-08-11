use poise::serenity_prelude as serenity;

#[derive(Debug, Clone)]
pub struct ComicPage {
    title: String,
    url: reqwest::Url,
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

    pub fn url(&self) -> &reqwest::Url {
        &self.url
    }

    pub fn slug(&self) -> &str {
        self.url()
            .path()
            .split_once('/')
            .expect("should have an extra slash")
            .1
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
    const HOST: &str = "nortverse.com";
    const COMICS_PATH: &str = "comic";
    const RANDOM_PATH: &str = "random";

    fn comic_url(slug: &str) -> reqwest::Url {
        reqwest::Url::parse(&format!(
            "https://{host}/{path}/{slug}",
            host = Self::HOST,
            path = Self::COMICS_PATH
        ))
        .expect("url should be well-formed")
    }

    pub async fn random(client: &reqwest::Client) -> reqwest::Result<Self> {
        let seed: u64 = rand::random();

        let response = client
            .get(format!(
                "https://{host}/{path}?r={seed}",
                host = Self::HOST,
                path = Self::RANDOM_PATH
            ))
            .send()
            .await?;

        let slug = response
            .url()
            .path_segments()
            .expect("should have path")
            .nth(1)
            .expect("should have 2nd item");

        Self::from_slug(client, slug).await
    }

    pub async fn from_slug(
        client: &reqwest::Client,
        slug: &str,
    ) -> std::result::Result<Self, reqwest::Error> {
        let url = Self::comic_url(slug);

        let text = client
            .get(url.as_str())
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

    pub async fn from_homepage(
        client: &reqwest::Client,
    ) -> std::result::Result<Self, reqwest::Error> {
        let homepage = reqwest::Url::parse(&format!("https://{host}", host = Self::HOST))
            .expect("url should be well-formed");

        let text = client
            .get(homepage)
            .send()
            .await?
            .text()
            .await
            .expect("page should have text");

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
                reqwest::Url::parse(
                    element
                        .attr("href")
                        .expect("a element should have href attr"),
                )
                .expect("href should be valid url"),
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
