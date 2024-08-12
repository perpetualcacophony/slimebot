use std::fmt::{Display, Write};

use super::comic::ComicPage;

#[derive(Copy, Clone, Debug)]
pub struct ResponseBuilder<'nort> {
    comic: &'nort ComicPage,
    in_guild: bool,
    include_date: bool,
    subscribed: bool,
}

impl<'nort> ResponseBuilder<'nort> {
    pub fn new(comic: &'nort ComicPage) -> Self {
        Self {
            comic,
            in_guild: false,
            include_date: true,
            subscribed: false,
        }
    }

    pub fn in_guild(mut self, value: bool) -> Self {
        self.in_guild = value;
        self
    }

    pub fn include_date(mut self, value: bool) -> Self {
        self.include_date = value;
        self
    }

    pub fn subscribed(mut self, value: bool) -> Self {
        self.subscribed = value;
        self
    }

    pub async fn attachments(
        &self,
        http: &poise::serenity_prelude::Http,
    ) -> poise::serenity_prelude::Result<
        impl Iterator<Item = poise::serenity_prelude::CreateAttachment> + '_,
    > {
        Ok(self.comic.attachments(http).await?.map(|mut attachment| {
            if self.in_guild {
                attachment.filename = format!("SPOILER_{}", attachment.filename);
            }

            attachment
        }))
    }

    pub async fn build_reply(
        self,
        http: &poise::serenity_prelude::Http,
    ) -> poise::serenity_prelude::Result<poise::CreateReply> {
        let mut builder = poise::CreateReply::default().content(self.to_string());

        builder = self
            .attachments(http)
            .await?
            .fold(builder, |builder, attachment| {
                builder.attachment(attachment)
            });

        Ok(builder)
    }

    pub async fn build_message(
        self,
        http: &poise::serenity_prelude::Http,
    ) -> poise::serenity_prelude::Result<poise::serenity_prelude::CreateMessage> {
        Ok(poise::serenity_prelude::CreateMessage::new()
            .content(self.to_string())
            .files(self.attachments(http).await?))
    }

    pub fn build_embed(self) -> poise::serenity_prelude::CreateEmbed {
        todo!()
    }
}

impl Display for ResponseBuilder<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.subscribed {
            f.write_str("New comic! (`..nortverse unsubscribe`) to unsubscribe")?;
        }

        write!(
            f,
            "## [{title}]({url})",
            title = self.comic.title(),
            url = self.comic.url()
        )?;

        if self.include_date {
            let date = format!("Posted {}", self.comic.date());
            f.write_char('\n')?;
            f.write_str(&date)?;
        }

        if self.in_guild {
            f.write_char('\n')?;
            f.write_str("`spoilered for potential NSFW`")?;
        }

        Ok(())
    }
}
