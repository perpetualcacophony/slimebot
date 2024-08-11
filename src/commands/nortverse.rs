use crate::{utils::poise::CommandResult, Context};

mod data;
use data::NortverseDataAsync;

mod error;
pub use error::NortverseError as Error;

mod comic;
use comic::ComicPage;

type Result<T, E = Error> = std::result::Result<T, E>;

#[tracing::instrument(skip_all)]
#[poise::command(
    slash_command,
    prefix_command,
    discard_spare_arguments,
    required_bot_permissions = "SEND_MESSAGES | VIEW_CHANNEL"
)]
pub async fn nortverse(ctx: Context<'_>) -> crate::Result<()> {
    let result: CommandResult = try {};

    result?;
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

    const HOST: &str = "nortverse.com";
    const COMICS_PATH: &str = "comic";

    fn comic_url(slug: &str) -> reqwest::Url {
        reqwest::Url::parse(&format!(
            "https://{host}/{path}/{slug}",
            host = Self::HOST,
            path = Self::COMICS_PATH
        ))
        .unwrap()
    }
}

impl<Data> Nortverse<Data>
where
    Data: NortverseDataAsync,
{
    async fn refresh_latest(&self) -> Result<(ComicPage, bool)> {
        let latest = ComicPage::from_homepage().await?;

        let data_slug = {
            let data = self.data().await;
            let data_slug = data.latest_slug().await.map_err(Error::data)?;
            data_slug.map(|as_ref| as_ref.as_ref().to_owned())
        };

        let updated = Some(latest.slug()) == data_slug.as_deref();

        if updated {
            self.data_mut()
                .await
                .set_latest(latest.slug().to_owned())
                .await
                .map_err(Error::data)?;
        }

        Ok((latest, updated))
    }
}
