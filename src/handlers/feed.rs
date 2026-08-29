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

use crate::models::{audios_dir, Channel, Episode};
use crate::utils::middleware::SessionOrBasicAuth;
use std::path::Path as FsPath;

use super::AppState;

#[derive(Deserialize)]
struct Info {
    key: String,
}

pub fn web_feed(cfg: &mut ServiceConfig) {
    cfg.service(
        web::resource("/channels/{key}/feed.xml")
            .route(web::get().to(get_feed))
            .wrap(SessionOrBasicAuth),
    )
    .service(
        web::resource("/{key}/feed.xml")
            .route(web::get().to(get_legacy_feed))
            .wrap(SessionOrBasicAuth),
    )
    .service(
        web::resource("/feed.xml")
            .route(web::get().to(get_global_feed))
            .wrap(SessionOrBasicAuth),
    );
}

async fn get_global_feed(data: Data<AppState>) -> impl Responder {
    info!("get_global_feed");
    let url = &data.config.url;
    match Episode::read_all_with_channels(&data.pool).await {
        Ok(episodes) => {
            let items = global_items(
                url,
                FsPath::new(audios_dir()),
                episodes,
                data.config.sponsorblock_enabled,
            );
            let link = format!("{url}/rss");
            let itunes = ITunesChannelExtensionBuilder::default()
                .summary(Some("All episodes from all channels, newest first".to_string()))
                .build();
            let channel_builder = ChannelBuilder::default()
                .title("All Episodes")
                .description("All episodes from all channels, newest first")
                .link(&link)
                .itunes_ext(Some(itunes))
                .items(items)
                .build();
            HttpResponse::Ok()
                .append_header(("Content-type", "application/rss+xml; charset=utf-8"))
                .body(channel_builder.to_string())
        }
        Err(e) => {
            error!("Error: {e}");
            HttpResponse::InternalServerError().body("Internal server error")
        }
    }
}

async fn get_feed(data: Data<AppState>, path: Path<Info>) -> impl Responder {
    info!("get_feed");
    let key = path.key.clone();
    match Channel::read_by_id_or_slug(&data.pool, &key).await {
        Ok(channel) => build_feed(&data, channel).await,
        Err(e) => {
            error!("Error: {e}");
            HttpResponse::NotFound().body("Feed not found")
        }
    }
}

async fn get_legacy_feed(data: Data<AppState>, path: Path<Info>) -> impl Responder {
    info!("get_legacy_feed");
    let key = path.key.clone();
    match Channel::read_by_slug(&data.pool, &key).await {
        Ok(channel) => build_feed(&data, channel).await,
        Err(e) => {
            error!("Error: {e}");
            HttpResponse::NotFound().body("Feed not found")
        }
    }
}

async fn build_feed(data: &Data<AppState>, channel: Channel) -> HttpResponse {
    let url = &data.config.url;
    match Episode::read_episodes_for_channel(&data.pool, channel.id).await {
        Ok(episodes) => {
            let items = channel_items(
                url,
                FsPath::new(audios_dir()),
                &channel.slug,
                episodes,
                data.config.sponsorblock_enabled,
            );
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
            HttpResponse::Ok()
                .append_header(("Content-type", "application/rss+xml; charset=utf-8"))
                .body(channel_builder.to_string())
        }
        Err(e) => {
            error!("Error: {e}");
            HttpResponse::InternalServerError().body("Internal server error")
        }
    }
}

fn episode_item(
    url: &str,
    audio_root: &FsPath,
    slug: &str,
    episode: Episode,
    title: String,
    sponsorblock_enabled: bool,
) -> rss::Item {
    let selected = episode.selected_media(&audio_root.join(slug), sponsorblock_enabled);
    let enclosure_url = format!("{url}/media/{slug}/{}", selected.filename);
    let description = format!("{}\n\n{}", episode.webpage_url, episode.description);
    let itunes = ITunesItemExtensionBuilder::default()
        .image(Some(episode.image))
        .summary(Some(description.clone()))
        .explicit(Some("No".to_string()))
        .episode_type(Some("Full".to_string()))
        .duration(Some(selected.duration))
        .build();
    let enclosure = EnclosureBuilder::default()
        .url(enclosure_url)
        .mime_type("audio/mpeg".to_string())
        .build();
    ItemBuilder::default()
        .title(Some(title))
        .description(Some(description))
        .enclosure(Some(enclosure))
        .guid(Some(GuidBuilder::default().value(episode.yt_id).build()))
        .pub_date(Some(episode.published_at.to_rfc2822()))
        .itunes_ext(Some(itunes))
        .build()
}

fn channel_items(
    url: &str,
    audio_root: &FsPath,
    slug: &str,
    episodes: Vec<Episode>,
    sponsorblock_enabled: bool,
) -> Vec<rss::Item> {
    episodes
        .into_iter()
        .map(|episode| {
            let title = episode.title.clone();
            episode_item(url, audio_root, slug, episode, title, sponsorblock_enabled)
        })
        .collect()
}

fn global_items(
    url: &str,
    audio_root: &FsPath,
    episodes: Vec<Episode>,
    sponsorblock_enabled: bool,
) -> Vec<rss::Item> {
    episodes
        .into_iter()
        .filter(|episode| !episode.channel_slug.is_empty())
        .map(|episode| {
            let slug = episode.channel_slug.clone();
            let title = if episode.channel_title.is_empty() {
                episode.title.clone()
            } else {
                format!("{}: {}", episode.channel_title, episode.title)
            };
            episode_item(url, audio_root, &slug, episode, title, sponsorblock_enabled)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::SponsorBlockCache;
    use chrono::{Duration, Utc};
    use sqlx::{migrate::Migrator, sqlite::SqlitePoolOptions};

    async fn fixture() -> (sqlx::SqlitePool, i64, std::path::PathBuf) {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        Migrator::new(FsPath::new(env!("CARGO_MANIFEST_DIR")).join("migrations"))
            .await
            .unwrap()
            .run(&pool)
            .await
            .unwrap();
        let now = Utc::now();
        let channel_id: i64 = sqlx::query_scalar(
            "INSERT INTO channels (url, title, slug, active, description, image, first, max, created_at, updated_at) \
             VALUES ('https://example.com', 'Channel title', 'channel_slug', TRUE, '', '', $1, 5, $1, $1) RETURNING id",
        )
        .bind(now)
        .fetch_one(&pool)
        .await
        .unwrap();
        for (index, yt_id) in ["processed", "empty", "missing"].iter().enumerate() {
            sqlx::query(
                "INSERT INTO episodes (channel_id, title, yt_id, webpage_url, published_at, duration, created_at, updated_at) \
                 VALUES ($1, $2, $3, $4, $5, '600', $5, $5)",
            )
            .bind(channel_id)
            .bind(format!("Episode {yt_id}"))
            .bind(yt_id)
            .bind(format!("https://example.com/{yt_id}"))
            .bind(now - Duration::seconds(index as i64))
            .execute(&pool)
            .await
            .unwrap();
        }
        let root = std::env::temp_dir().join(format!("u2vpodcast-feed-{}", rand::random::<u64>()));
        std::fs::create_dir_all(root.join("channel_slug")).unwrap();
        (pool, channel_id, root)
    }

    #[actix_web::test]
    async fn simulated_feeds_select_processed_media_and_fallbacks() {
        let (pool, channel_id, root) = fixture().await;
        let episodes = Episode::read_episodes_for_channel(&pool, channel_id).await.unwrap();
        let by_id = |yt_id: &str| episodes.iter().find(|episode| episode.yt_id == yt_id).unwrap().id;
        SponsorBlockCache::upsert_success(
            &pool,
            by_id("processed"),
            &[],
            "processed-hash",
            "processed-hash",
            Some("processed.sponsorblock.abcdef.mp3"),
            Some(540.0),
        )
        .await
        .unwrap();
        SponsorBlockCache::upsert_success(
            &pool, by_id("empty"), &[], "empty-hash", "empty-hash", None, None,
        )
            .await
            .unwrap();
        SponsorBlockCache::upsert_success(
            &pool,
            by_id("missing"),
            &[],
            "missing-hash",
            "missing-hash",
            Some("missing.sponsorblock.abcdef.mp3"),
            Some(500.0),
        )
        .await
        .unwrap();
        std::fs::write(
            root.join("channel_slug/processed.sponsorblock.abcdef.mp3"),
            b"fixture",
        )
        .unwrap();

        let channel_episodes = Episode::read_episodes_for_channel(&pool, channel_id).await.unwrap();
        let items = channel_items(
            "http://backend", &root, "channel_slug", channel_episodes, true,
        );
        let processed = items.iter().find(|item| item.guid().unwrap().value() == "processed").unwrap();
        assert!(processed.enclosure().unwrap().url().ends_with("/processed.sponsorblock.abcdef.mp3"));
        assert_eq!(processed.itunes_ext().unwrap().duration(), Some("540"));
        for yt_id in ["empty", "missing"] {
            let item = items.iter().find(|item| item.guid().unwrap().value() == yt_id).unwrap();
            assert!(item.enclosure().unwrap().url().ends_with(&format!("/{yt_id}.mp3")));
            assert_eq!(item.itunes_ext().unwrap().duration(), Some("600"));
        }

        let global = global_items(
            "http://backend",
            &root,
            Episode::read_all_with_channels(&pool).await.unwrap(),
            true,
        );
        assert_eq!(global.len(), 3);
        assert!(global.iter().all(|item| item.title().unwrap().starts_with("Channel title: ")));
        assert_eq!(processed.guid().unwrap().value(), "processed");

        let disabled_channel = channel_items(
            "http://backend",
            &root,
            "channel_slug",
            Episode::read_episodes_for_channel(&pool, channel_id).await.unwrap(),
            false,
        );
        let disabled = disabled_channel
            .iter()
            .find(|item| item.guid().unwrap().value() == "processed")
            .unwrap();
        assert!(disabled.enclosure().unwrap().url().ends_with("/processed.mp3"));
        assert_eq!(disabled.itunes_ext().unwrap().duration(), Some("600"));

        let disabled_global = global_items(
            "http://backend",
            &root,
            Episode::read_all_with_channels(&pool).await.unwrap(),
            false,
        );
        let disabled = disabled_global
            .iter()
            .find(|item| item.guid().unwrap().value() == "processed")
            .unwrap();
        assert!(disabled.enclosure().unwrap().url().ends_with("/processed.mp3"));
        assert_eq!(disabled.itunes_ext().unwrap().duration(), Some("600"));
        std::fs::remove_dir_all(root).unwrap();
    }
}
