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
}

impl ComicPage {
    pub async fn from_slug(slug: &str) -> std::result::Result<Self, reqwest::Error> {
        let url = super::Nortverse::<()>::comic_url(slug);

        let text = reqwest::get(url.as_str())
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

    pub async fn from_homepage() -> std::result::Result<Self, reqwest::Error> {
        let homepage = reqwest::Url::parse(&format!(
            "https://{host}",
            host = super::Nortverse::<()>::HOST
        ))
        .unwrap();

        let text = reqwest::get(homepage)
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
