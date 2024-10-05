use dynasty2::Dynasty;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct Series {
    slug: dynasty2::SlugOwned,
    schedule: Option<Schedule>,
}

impl Series {
    pub fn slug(&self) -> dynasty2::Slug {
        self.slug.as_ref()
    }

    pub async fn latest_chapter(&self, dynasty: &Dynasty) -> dynasty2::Result<String> {
        Ok(dynasty
            .series(&self.slug())?
            .await?
            .chapters()
            .last()
            .expect("should have at least one chapter")
            .slug()
            .to_string())
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub enum Schedule {
    Weekly { days: Vec<u8> },
}
