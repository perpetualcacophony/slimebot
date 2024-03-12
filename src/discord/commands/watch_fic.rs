use std::{fs, time::Duration};

use anyhow::{anyhow, bail};
use poise::serenity_prelude::{Channel, Role};
use scraper::{Html, Selector};
use tracing::{error, info, instrument};

use super::Error;
use crate::Data;

type Context<'a> = poise::Context<'a, Data, Error>;

#[poise::command(slash_command)]
pub async fn watch_fic(
    ctx: Context<'_>,
    id: usize,
    channel: Channel,
    role: Role,
) -> super::CommandResult {
    /*let reply = ctx
        .send(|f| {
            f.content("boop").components(|f| {
                f.create_action_row(|f| f.create_button(|b| b.label("oop").custom_id("foo")))
            })
        })
        .await?;
    */

    // to do!! make this little menu work
    /*
    let interaction = reply
        .message()
        .await?
        .await_component_interaction(ctx)    async fn _ping(ctx: Context<'_>) -> anyhow::Result<()> {

    }
        .author_id(ctx.author().id)
        .await;
    */

    loop {
        let stored_chapter_count =
            read_chapter_count(id).expect("stored chapter count should be valid");
        let chapter_ids = get_chapter_ids(id)
            .await
            .expect("getting chapter ids should not fail");

        if stored_chapter_count < chapter_ids.len() {
            info!("request made. update!");
            store_chapter_count(id, chapter_ids.len())
                .expect("storing chapter count should not fail");

            channel
                .id()
                .say(
                    &ctx.http(),
                    format!(
                        "<@&{}> **Intertwined has updated!**
                    chapter {}: https://archiveofourown.org/works/{}/chapters/{}",
                        role.id,
                        chapter_ids.len(),
                        id,
                        chapter_ids
                            .last()
                            .expect("work should have at least 1 chapter")
                    ),
                )
                .await
                .expect("sending message should not fail");
        } else {
            info!("request made. no update")
        }

        std::thread::sleep(Duration::from_secs(300));
    }

    //Ok(())
}

fn _has_updated(work_id: usize, current_chapter_count: usize) -> Result<bool, Error> {
    let stored_chapter_count: usize = read_chapter_count(work_id)?;

    //let current_chapter_count = get_chapter_count(work_id)?;

    match stored_chapter_count.cmp(&current_chapter_count) {
        std::cmp::Ordering::Less => Ok(true),
        std::cmp::Ordering::Equal => Ok(false),
        std::cmp::Ordering::Greater => Err(anyhow!("chapter count not stored properly").into()),
    }
}

// i've removed a few more performant ao3 hooks in favor of this one
// i'm a big fan of ao3 and since they don't have an api i want to minimize expensive calls
// it's a little bit of runtime overhead but nbd
#[instrument(level = "TRACE")]
async fn get_chapter_ids(work_id: usize) -> Result<Vec<usize>, anyhow::Error> {
    let work_index = format!("https://archiveofourown.org/works/{work_id}/navigate");

    let Ok(html) = reqwest::get(work_index).await else {
        error!("d");
        bail!("")
    };

    let Ok(text) = html.text().await else {
        error!("d");
        bail!("f")
    };

    //.text()
    //.await
    //.expect("ao3 request failed");

    let doc = Html::parse_document(&text);
    let selector = Selector::parse("ol.chapter.index.group>li>a")
        .expect("hard-coded selector should be valid");

    let chapter_ids = doc
        .select(&selector)
        .map(|el| {
            el.value()
                .attr("href")
                .expect("<a> element should have href attr")
                .split("/chapters/")
                .nth(1)
                .expect("should have at least 2 chapters")
                .parse()
                .expect("id should be valid usize")
        })
        .collect::<Vec<usize>>();

    Ok(chapter_ids)
}
/* */
// this method's annoying to work with
// it's tied to, like, an explicit work right? why supply the chapter count?
// if i could work with an api, i probably *would* have this func call ao3
// but. i can't. i think i've minimized ao3 calls to, like, 1 every loop
fn store_chapter_count(work_id: usize, chapter_count: usize) -> Result<(), Error> {
    //fs::write(format!("works/{work_id}.len"), chapter_count.to_string())?;

    Ok(())
}

fn read_chapter_count(work_id: usize) -> Result<usize, Error> {
    //Ok(fs::read_to_string(format!("works/{}.len", work_id))?.parse()?)
    Ok(0)
}
