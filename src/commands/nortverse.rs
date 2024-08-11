use crate::{utils::poise::CommandResult, Context};

mod data;
use data::NortverseDataAsync;

mod error;
pub use error::NortverseError as Error;

type Result<T, Data> = std::result::Result<T, Error<Data>>;

#[tracing::instrument(skip_all)]
#[poise::command(
    slash_command,
    prefix_command,
    discard_spare_arguments,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
pub async fn nortverse(ctx: Context<'_>) -> crate::Result<()> {
    Ok(())
}

#[derive(Debug)]
struct Nortverse<Data = data::MongoDb> {
    data: std::sync::Arc<tokio::sync::RwLock<Data>>,
}

impl<T> Clone for Nortverse<T> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
        }
    }
}

impl<Data> Nortverse<Data> {
    async fn data(&self) -> tokio::sync::RwLockReadGuard<Data> {
        self.data.read().await
    }

    async fn data_mut(&self) -> tokio::sync::RwLockWriteGuard<Data> {
        self.data.write().await
    }
}

impl<Data> Nortverse<Data>
where
    Data: NortverseDataAsync,
{
    const HOST: &str = "nortverse.com";
    const COMICS_PATH: &str = "comic";

    async fn comic_url(slug: &str) -> reqwest::Url {
        reqwest::Url::parse(&format!(
            "https://{host}/{path}/{slug}",
            host = Self::HOST,
            path = Self::COMICS_PATH
        ))
        .unwrap()
    }

    async fn latest(&self) -> Result<reqwest::Url, Data::Error> {
        let text = reqwest::get(format!("https://{host}", host = Self::HOST))
            .await?
            .text()
            .await?;

        let html = scraper::Html::parse_document(&text);
        let selector =
            scraper::Selector::parse(".entry-title>a").expect("should be a valid selector");

        Ok(reqwest::Url::parse(
            html.select(&selector)
                .map(|element| {
                    element
                        .attr("href")
                        .expect("a element should have href attr")
                })
                .next()
                .expect("homepage should have a title with a link"),
        )
        .expect("href should be valid url"))
    }
}
