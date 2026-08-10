use actix_web::{
    web::{self, Data, Path, ServiceConfig},
    HttpResponse, Responder,
};
use rss::{
    extension::itunes::{ITunesChannelExtensionBuilder, ITunesItemExtensionBuilder},
    ChannelBuilder, EnclosureBuilder, GuidBuilder, ItemBuilder,
};
use serde::Deserialize;
use tracing::{error, info};

use crate::models::{Channel, Episode};
use crate::utils::middleware::BasicAuthGuard;

use super::AppState;

#[derive(Deserialize)]
struct Info {
    slug: String,
}

pub fn web_feed(cfg: &mut ServiceConfig) {
    cfg.service(
        web::resource("/channels/{slug}/feed.xml")
            .route(web::get().to(get_feed))
            .wrap(BasicAuthGuard),
    );
}

async fn get_feed(data: Data<AppState>, path: Path<Info>) -> impl Responder {
    info!("get_feed");
    let config = &data.config;
    let url = &config.url;
    let slug = path.slug.clone();
    match Channel::read_by_slug(&data.pool, &slug).await {
        Ok(channel) => match Episode::read_episodes_for_channel(&data.pool, channel.id).await {
            Ok(episodes) => {
                let mut items = Vec::new();
                for episode in episodes {
                    let yt_id = &episode.yt_id;
                    let enclosure = format!("{url}/media/{slug}/{yt_id}.mp3");
                    let itunes = ITunesItemExtensionBuilder::default()
                        .image(Some(episode.image))
                        .summary(Some(episode.description.to_string()))
                        .explicit(Some("No".to_string()))
                        .episode_type(Some("Full".to_string()))
                        .duration(Some(episode.duration))
                        .build();
                    let enclosure = EnclosureBuilder::default()
                        .url(&enclosure)
                        .mime_type("audio/mpeg".to_string())
                        .build();
                    let guid = GuidBuilder::default().value(episode.yt_id).build();
                    let item = ItemBuilder::default()
                        .title(Some(episode.title))
                        .description(Some(episode.description))
                        .enclosure(Some(enclosure))
                        .guid(Some(guid))
                        .pub_date(Some(episode.published_at.to_string()))
                        .itunes_ext(Some(itunes))
                        .build();
                    items.push(item);
                }
                let link = format!("{url}/rss");
                let itunes = ITunesChannelExtensionBuilder::default()
                    .image(Some(channel.image))
                    .summary(Some(channel.description.clone()))
                    .build();
                let channel_builder = ChannelBuilder::default()
                    .title(channel.title)
                    .description(channel.description)
                    .link(&link)
                    .itunes_ext(Some(itunes))
                    .items(items)
                    .build();
                Ok(HttpResponse::Ok()
                    .append_header(("Content-type", "application/rss+xml; charset=utf-8"))
                    .body(channel_builder.to_string()))
            }
            Err(e) => {
                error!("Error: {e}");
                Err(e)
            }
        },
        Err(e) => {
            error!("Error: {e}");
            Err(e)
        }
    }
}
