use super::{Error, PlaylistItem, SponsorBlockSegment};
use actix_web::http::StatusCode;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{
    query,
    sqlite::{SqlitePool, SqliteRow},
    Row,
};
use std::path::Path;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Episode {
    pub id: i64,
    pub channel_id: i64,
    #[serde(default)]
    pub channel_slug: String,
    #[serde(default = "get_default_empty")]
    pub channel_title: String,
    pub title: String,
    #[serde(default = "get_default_empty")]
    pub description: String,
    pub yt_id: String,
    pub webpage_url: String,
    pub published_at: DateTime<Utc>,
    pub duration: String,
    #[serde(default = "get_default_empty")]
    pub image: String,
    pub listen: bool,
    pub position_seconds: i64,
    pub listened_at: Option<DateTime<Utc>>,
    pub favorite: bool,
    #[serde(default)]
    pub sponsorblock_segments: Vec<SponsorBlockSegment>,
    #[serde(default)]
    pub sponsorblock_hash: Option<String>,
    #[serde(skip)]
    pub sponsorblock_processed_filename: Option<String>,
    #[serde(skip)]
    pub sponsorblock_processed_duration: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Leaf payload with just the playback-progress fields, used by the dedicated
/// progress endpoints so clients never receive the full episode row.
#[derive(Debug, Clone, Serialize)]
pub struct EpisodeProgress {
    pub id: i64,
    pub yt_id: String,
    pub position_seconds: i64,
    pub listen: bool,
    pub listened_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SelectedEpisodeMedia {
    pub filename: String,
    pub duration: String,
}

fn get_default_empty() -> String {
    "".to_string()
}

impl Episode {
    pub fn selected_media(&self, channel_dir: &Path) -> SelectedEpisodeMedia {
        if let (Some(filename), Some(duration)) = (
            self.sponsorblock_processed_filename.as_deref(),
            self.sponsorblock_processed_duration,
        ) {
            if channel_dir.join(filename).is_file() {
                return SelectedEpisodeMedia {
                    filename: filename.to_string(),
                    duration: duration.round().max(0.0).to_string(),
                };
            }
        }
        SelectedEpisodeMedia {
            filename: format!("{}.mp3", self.yt_id),
            duration: self.duration.clone(),
        }
    }

    fn sponsorblock_fields(
        row: &SqliteRow,
    ) -> (Vec<SponsorBlockSegment>, Option<String>, Option<String>, Option<f64>) {
        let segments = row
            .try_get::<Option<String>, _>("sponsorblock_segments_json")
            .ok()
            .flatten()
            .and_then(|json| serde_json::from_str(&json).ok())
            .unwrap_or_default();
        let hash = row.try_get("sponsorblock_hash").unwrap_or(None);
        let filename = row.try_get("sponsorblock_processed_filename").unwrap_or(None);
        let duration = row.try_get("sponsorblock_processed_duration").unwrap_or(None);
        (segments, hash, filename, duration)
    }

    fn from_row(row: SqliteRow) -> Self {
        info!("from_row");
        let (segments, hash, filename, processed_duration) = Self::sponsorblock_fields(&row);
        Self {
            id: row.get("id"),
            channel_id: row.get("channel_id"),
            channel_slug: String::new(),
            channel_title: String::new(),
            title: row.get("title"),
            description: row.get("description"),
            yt_id: row.get("yt_id"),
            webpage_url: row.get("webpage_url"),
            published_at: row.get("published_at"),
            duration: row.get("duration"),
            image: row.get("image"),
            listen: row.get("listen"),
            position_seconds: row.get("position_seconds"),
            listened_at: row.get("listened_at"),
            favorite: row.get("favorite"),
            sponsorblock_segments: segments,
            sponsorblock_hash: hash,
            sponsorblock_processed_filename: filename,
            sponsorblock_processed_duration: processed_duration,
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    pub(crate) fn from_row_with_channel(row: SqliteRow) -> Self {
        info!("from_row_with_channel");
        let (segments, hash, filename, processed_duration) = Self::sponsorblock_fields(&row);
        Self {
            id: row.get("id"),
            channel_id: row.get("channel_id"),
            channel_slug: row.get("channel_slug"),
            channel_title: row.get("channel_title"),
            title: row.get("title"),
            description: row.get("description"),
            yt_id: row.get("yt_id"),
            webpage_url: row.get("webpage_url"),
            published_at: row.get("published_at"),
            duration: row.get("duration"),
            image: row.get("image"),
            listen: row.get("listen"),
            position_seconds: row.get("position_seconds"),
            listened_at: row.get("listened_at"),
            favorite: row.get("favorite"),
            sponsorblock_segments: segments,
            sponsorblock_hash: hash,
            sponsorblock_processed_filename: filename,
            sponsorblock_processed_duration: processed_duration,
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn new(pool: &SqlitePool, channel_id: i64, title: &str,
            description: &str, yt_id: &str, webpage_url: &str,
            published_at: &DateTime<Utc>, duration: &str, image: &str,
            listen: bool) -> Result<Self, Error>{
        info!("new");
        let created_at = Utc::now();
        let updated_at = created_at;
        let mut episode = Self {
            id: -1,
            channel_id,
            channel_slug: String::new(),
            channel_title: String::new(),
            title: title.to_string(),
            description: description.to_string(),
            yt_id: yt_id.to_string(),
            webpage_url: webpage_url.to_string(),
            published_at: *published_at,
            duration: duration.to_string(),
            image: image.to_string(),
            listen,
            position_seconds: 0,
            listened_at: None,
            favorite: false,
            sponsorblock_segments: Vec::new(),
            sponsorblock_hash: None,
            sponsorblock_processed_filename: None,
            sponsorblock_processed_duration: None,
            created_at,
            updated_at,
        };
        episode.save(pool).await
    }

    pub async fn create(
        pool: &SqlitePool,
        episode: &Self,
    ) -> Result<Episode, Error> {
        let sql = "INSERT INTO episodes (channel_id, title, description, yt_id,
                   webpage_url, published_at, duration, image, listen,
                   position_seconds, listened_at, favorite, created_at,
                   updated_at)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12,
                   $13, $14) RETURNING *;";
        query(sql)
            .bind(episode.channel_id)
            .bind(&episode.title)
            .bind(&episode.description)
            .bind(&episode.yt_id)
            .bind(&episode.webpage_url)
            .bind(episode.published_at)
            .bind(&episode.duration)
            .bind(&episode.image)
            .bind(episode.listen)
            .bind(episode.position_seconds)
            .bind(episode.listened_at)
            .bind(episode.favorite)
            .bind(episode.created_at)
            .bind(episode.updated_at)
            .map(Self::from_row)
            .fetch_one(pool)
            .await
            .map_err(|e| e.into())
    }

    pub async fn read(pool: &SqlitePool, id: i64) -> Result<Self, Error>{
        info!("read");
        let sql = "SELECT e.*, sc.segments_json AS sponsorblock_segments_json, \
                  sc.snapshot_hash AS sponsorblock_hash, \
                  sc.processed_filename AS sponsorblock_processed_filename, \
                  sc.processed_duration AS sponsorblock_processed_duration \
               FROM episodes e LEFT JOIN sponsorblock_cache sc ON sc.episode_id = e.id \
               WHERE e.id = $1";
        query(sql)
            .bind(id)
            .map(Self::from_row)
            .fetch_one(pool)
            .await
            .map_err(|e| Error::new_with_status_code(&e.to_string(), StatusCode::NOT_FOUND))
    }

    pub async fn read_episodes_for_channel(pool: &SqlitePool, channel_id: i64) -> Result<Vec<Self>, Error>{
        info!("read_all");
        let sql = "SELECT e.*, sc.segments_json AS sponsorblock_segments_json, \
                  sc.snapshot_hash AS sponsorblock_hash, \
                  sc.processed_filename AS sponsorblock_processed_filename, \
                  sc.processed_duration AS sponsorblock_processed_duration \
               FROM episodes e LEFT JOIN sponsorblock_cache sc ON sc.episode_id = e.id \
               WHERE e.channel_id = $1 ORDER BY e.published_at DESC";
        query(sql)
            .bind(channel_id)
            .map(Self::from_row)
            .fetch_all(pool)
            .await
            .map_err(|e| e.into())
    }
    // Unused today (handlers use `read_all_with_channels`); kept as the plain
    // variant of the same query, following the `#[allow(unused)]` precedent
    // used on other optional model helpers.
    #[allow(dead_code)]
    pub async fn read_all(pool: &SqlitePool) -> Result<Vec<Self>, Error>{
        info!("read_all");
        let sql = "SELECT e.*, sc.segments_json AS sponsorblock_segments_json, \
                  sc.snapshot_hash AS sponsorblock_hash, \
                  sc.processed_filename AS sponsorblock_processed_filename, \
                  sc.processed_duration AS sponsorblock_processed_duration \
               FROM episodes e LEFT JOIN sponsorblock_cache sc ON sc.episode_id = e.id \
               ORDER BY e.published_at DESC";
        query(sql)
            .map(Self::from_row)
            .fetch_all(pool)
            .await
            .map_err(|e| e.into())
    }

    pub async fn read_all_with_channels(pool: &SqlitePool) -> Result<Vec<Self>, Error>{
        info!("read_all_with_channels");
        let sql = "SELECT e.*, COALESCE(c.slug, '') AS channel_slug, COALESCE(c.title, '') AS channel_title, \
                  sc.segments_json AS sponsorblock_segments_json, \
                  sc.snapshot_hash AS sponsorblock_hash, \
                  sc.processed_filename AS sponsorblock_processed_filename, \
                  sc.processed_duration AS sponsorblock_processed_duration \
                   FROM episodes e LEFT JOIN channels c ON c.id = e.channel_id \
               LEFT JOIN sponsorblock_cache sc ON sc.episode_id = e.id \
                   ORDER BY e.published_at DESC";
        query(sql)
            .map(Self::from_row_with_channel)
            .fetch_all(pool)
            .await
            .map_err(|e| e.into())
    }

    pub async fn read_by_yt_id_with_channel(
        pool: &SqlitePool,
        yt_id: &str,
    ) -> Result<Self, Error> {
        let sql = "SELECT e.*, COALESCE(c.slug, '') AS channel_slug, COALESCE(c.title, '') AS channel_title, \
                          sc.segments_json AS sponsorblock_segments_json, sc.snapshot_hash AS sponsorblock_hash, \
                          sc.processed_filename AS sponsorblock_processed_filename, \
                          sc.processed_duration AS sponsorblock_processed_duration \
                   FROM episodes e LEFT JOIN channels c ON c.id = e.channel_id \
                   LEFT JOIN sponsorblock_cache sc ON sc.episode_id = e.id \
                   WHERE e.yt_id = $1 ORDER BY e.id LIMIT 1";
        query(sql)
            .bind(yt_id)
            .map(Self::from_row_with_channel)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| Error::new_with_status_code("episode not found", StatusCode::NOT_FOUND))
    }

    pub async fn exists(pool: &SqlitePool, channel_id: i64, yt_id: &str) -> bool {
        let sql = "SELECT count(*) FROM episodes WHERE channel_id = $1 AND yt_id = $2";
        match query(sql)
            .bind(channel_id)
            .bind(yt_id)
            .map(|row: SqliteRow| -> i64 { row.get(0) })
            .fetch_one(pool)
            .await
        {
            Ok(value) => value > 0,
            Err(e) => {
                tracing::info!("Error on exists {}", e);
                false
            }
        }
    }

    #[allow(unused)]
    pub async fn count(pool: &SqlitePool, channel_id: i64) -> i64 {
        let sql = "SELECT count(*) FROM episodes WHERE channel_id = $1";
        match query(sql)
            .bind(channel_id)
            .map(|row: SqliteRow| -> i64 { row.get(0) })
            .fetch_one(pool)
            .await
        {
            Ok(value) => value,
            Err(e) => {
                tracing::info!("Error on count {}", e);
                0
            }
        }
    }


    #[allow(unused)]
    pub async fn read_with_pagination(
        pool: &SqlitePool,
        channel_id: i64,
        page: i64,
        per_page: i64,
    ) -> Result<Vec<Episode>, Error> {
        tracing::debug!(
            "Channel: {}. Página: {}. Páginas: {}",
            channel_id,
            page,
            per_page
        );
        // A malformed page (<= 0) must never yield a negative SQL OFFSET.
        let page = page.max(1);
        let offset = (page - 1) * per_page;
        let sql = "SELECT e.*, sc.segments_json AS sponsorblock_segments_json, \
                  sc.snapshot_hash AS sponsorblock_hash, \
                  sc.processed_filename AS sponsorblock_processed_filename, \
                  sc.processed_duration AS sponsorblock_processed_duration \
               FROM episodes e LEFT JOIN sponsorblock_cache sc ON sc.episode_id = e.id
               WHERE e.channel_id = $1 ORDER BY e.published_at DESC
                   LIMIT $2 OFFSET $3";
        query(sql)
            .bind(channel_id)
            .bind(per_page)
            .bind(offset)
            .map(Self::from_row)
            .fetch_all(pool)
            .await
            .map_err(|e| e.into())
    }

    pub async fn update(pool: &SqlitePool, episode: &Self) -> Result<Self, Error> {
        info!("update");
        let sql = "UPDATE episodes SET channel_id = $2, title = $3,
                   description = $4, yt_id = $5, published_at = $6,
                   duration = $7, image = $8, listen = $9,
                   position_seconds = $10, listened_at = $11, favorite = $12,
                   updated_at = $13
                   WHERE id = $1 RETURNING * ;";
        let updated_at = Utc::now();
        query(sql)
            .bind(episode.id)
            .bind(episode.channel_id)
            .bind(&episode.title)
            .bind(&episode.description)
            .bind(&episode.yt_id)
            .bind(episode.published_at)
            .bind(&episode.duration)
            .bind(&episode.image)
            .bind(episode.listen)
            .bind(episode.position_seconds)
            .bind(episode.listened_at)
            .bind(episode.favorite)
            .bind(updated_at)
            .map(Self::from_row)
            .fetch_one(pool)
            .await
            .map_err(|e| e.into())
    }

    /// Persists a single playback-progress write: the position (always) plus
    /// the listened mark when `listened` is true (clearing it when false).
    /// `listened_at` only changes on the false->true transition: re-saving an
    /// already-listened episode keeps the original completion timestamp, so it
    /// never drifts with routine position saves. Returns the refreshed episode
    /// row, or an error when no episode matches.
    pub async fn update_progress(
        pool: &SqlitePool,
        id: i64,
        position_seconds: i64,
        listened: bool,
    ) -> Result<Self, Error> {
        info!("update_progress");
        let listened_at = if listened { Some(Utc::now()) } else { None };
        let updated_at = Utc::now();
        let sql = "UPDATE episodes SET position_seconds = $2, listen = $3,
                   listened_at = CASE WHEN $3 THEN COALESCE(listened_at, $4) ELSE NULL END,
                   updated_at = $5
                   WHERE id = $1 RETURNING *;";
        match query(sql)
            .bind(id)
            .bind(position_seconds)
            .bind(listened)
            .bind(listened_at)
            .bind(updated_at)
            .map(Self::from_row)
            .fetch_optional(pool)
            .await
        {
            Ok(Some(episode)) => Ok(episode),
            Ok(None) => Err(Error::new_with_status_code(
                "episode not found",
                StatusCode::NOT_FOUND,
            )),
            Err(e) => Err(e.into()),
        }
    }

    /// Progress update keyed by the episode's public id (`yt_id`), the same
    /// identity the media URLs use. Resolves the row first because the only
    /// UNIQUE constraint is `(channel_id, yt_id)`; the write itself then goes
    /// through `update_progress` on the resolved row id.
    pub async fn update_progress_by_yt_id(
        pool: &SqlitePool,
        yt_id: &str,
        position_seconds: i64,
        listened: bool,
    ) -> Result<Self, Error> {
        info!("update_progress_by_yt_id");
        let id = query("SELECT id FROM episodes WHERE yt_id = $1 ORDER BY id LIMIT 1")
            .bind(yt_id)
            .map(|row: SqliteRow| row.get::<i64, _>("id"))
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| {
                Error::new_with_status_code("episode not found", StatusCode::NOT_FOUND)
            })?;
        Self::update_progress(pool, id, position_seconds, listened).await
    }

    /// Sets the favorite flag on the episode identified by its public id
    /// (`yt_id`), the same identity the progress and media endpoints use.
    /// `favorite` is written with its own targeted UPDATE so unrelated fields
    /// are never touched; missing episodes surface as 404, mirroring the
    /// progress endpoints.
    pub async fn set_favorite_by_yt_id(
        pool: &SqlitePool,
        yt_id: &str,
        favorite: bool,
    ) -> Result<Self, Error> {
        info!("set_favorite_by_yt_id");
        let updated_at = Utc::now();
        let sql = "UPDATE episodes SET favorite = $1, updated_at = $2 \
                   WHERE yt_id = $3 RETURNING *;";
        match query(sql)
            .bind(favorite)
            .bind(updated_at)
            .bind(yt_id)
            .map(Self::from_row)
            .fetch_optional(pool)
            .await
        {
            Ok(Some(episode)) => Ok(episode),
            Ok(None) => Err(Error::new_with_status_code(
                "episode not found",
                StatusCode::NOT_FOUND,
            )),
            Err(e) => Err(e.into()),
        }
    }

    /// Returns the stored playback-progress fields for an episode, keyed by its
    /// public id. Used by the player when starting playback so the resume
    /// decision uses the server's authoritative value instead of a stale copy.
    pub async fn read_progress_by_yt_id(
        pool: &SqlitePool,
        yt_id: &str,
    ) -> Result<EpisodeProgress, Error> {
        info!("read_progress_by_yt_id");
        let sql = "SELECT id, yt_id, position_seconds, listen, listened_at
                   FROM episodes WHERE yt_id = $1 ORDER BY id LIMIT 1;";
        query(sql)
            .bind(yt_id)
            .map(|row: SqliteRow| EpisodeProgress {
                id: row.get("id"),
                yt_id: row.get("yt_id"),
                position_seconds: row.get("position_seconds"),
                listen: row.get("listen"),
                listened_at: row.get("listened_at"),
            })
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| {
                Error::new_with_status_code("episode not found", StatusCode::NOT_FOUND)
            })
    }

    pub async fn remove(pool: &SqlitePool, id: i64) -> Result<Episode, Error> {
        info!("remove");
        let mut tx = pool.begin().await?;
        let sql = "DELETE from episodes WHERE id = $1 RETURNING * ;";
        let episode = query(sql)
            .bind(id)
            .map(Self::from_row)
            .fetch_one(&mut *tx)
            .await?;
        PlaylistItem::purge_episode(&mut tx, id).await?;
        tx.commit().await?;
        Ok(episode)
    }


    pub async fn save(&mut self, pool: &SqlitePool) -> Result<Self, Error>{
        info!("save");
        if self.id > -1 {
            let saved = Self::update(pool, self).await?;
            self.updated_at = saved.updated_at;
            Ok(saved)
        }else{
            let saved = Self::create(pool, self).await?;
            self.id = saved.id;
            Ok(saved)
        }
    }
}

#[cfg(test)]
mod episode_update_tests {
    use super::*;
    use sqlx::{
        sqlite::SqlitePoolOptions,
        migrate::Migrator,
    };
    use std::path::Path;

    async fn memory_pool() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("memory pool");
        let migrations = Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations");
        Migrator::new(migrations)
            .await
            .expect("load migrations")
            .run(&pool)
            .await
            .expect("run migrations");
        pool
    }

    async fn insert_channel(pool: &SqlitePool) -> i64 {
        let now = Utc::now();
        query(
            "INSERT INTO channels (url, title, slug, active, description, image, \
             first, max, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) RETURNING id",
        )
        .bind("https://example.com/ep-test")
        .bind("Episode Test Channel")
        .bind("ep_test_channel")
        .bind(true)
        .bind("")
        .bind("")
        .bind(now)
        .bind(5i64)
        .bind(now)
        .bind(now)
        .map(|row: SqliteRow| row.get::<i64, _>("id"))
        .fetch_one(pool)
        .await
        .expect("insert channel")
    }

    fn episode_struct(channel_id: i64, yt_id: &str) -> Episode {
        Episode {
            id: -1,
            channel_id,
            channel_slug: String::new(),
            channel_title: String::new(),
            title: format!("episode {yt_id}"),
            description: String::new(),
            yt_id: yt_id.to_string(),
            webpage_url: format!("https://youtu.be/{yt_id}"),
            published_at: Utc::now(),
            duration: "00:10:00".to_string(),
            image: String::new(),
            listen: false,
            position_seconds: 0,
            listened_at: None,
            favorite: false,
            sponsorblock_segments: Vec::new(),
            sponsorblock_hash: None,
            sponsorblock_processed_filename: None,
            sponsorblock_processed_duration: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn selected_media_requires_an_existing_processed_file() {
        let channel_dir = std::env::temp_dir().join(format!(
            "u2vpodcast-selected-media-{}",
            rand::random::<u64>()
        ));
        std::fs::create_dir_all(&channel_dir).expect("create fixture directory");
        let mut episode = episode_struct(1, "video-id");

        assert_eq!(
            episode.selected_media(&channel_dir),
            SelectedEpisodeMedia {
                filename: "video-id.mp3".to_string(),
                duration: "00:10:00".to_string(),
            }
        );

        episode.sponsorblock_hash = Some("empty-hash".to_string());
        assert_eq!(episode.selected_media(&channel_dir).filename, "video-id.mp3");

        episode.sponsorblock_processed_filename =
            Some("video-id.sponsorblock.abcdef.mp3".to_string());
        episode.sponsorblock_processed_duration = Some(539.6);
        assert_eq!(episode.selected_media(&channel_dir).filename, "video-id.mp3");

        std::fs::write(
            channel_dir.join("video-id.sponsorblock.abcdef.mp3"),
            b"fixture",
        )
        .expect("write processed fixture");
        assert_eq!(
            episode.selected_media(&channel_dir),
            SelectedEpisodeMedia {
                filename: "video-id.sponsorblock.abcdef.mp3".to_string(),
                duration: "540".to_string(),
            }
        );
        std::fs::remove_dir_all(channel_dir).expect("remove fixture directory");
    }

    #[tokio::test]
    async fn updating_an_existing_episode_affects_exactly_one_row() {
        let pool = memory_pool().await;
        let channel_id = insert_channel(&pool).await;

        let first = episode_struct(channel_id, "aaa111");
        let saved_1 = Episode::create(&pool, &first).await.expect("create ep 1");
        let second = episode_struct(channel_id, "bbb222");
        let _saved_2 = Episode::create(&pool, &second).await.expect("create ep 2");

        let mut update = saved_1.clone();
        update.title = "episode aaa111 (updated)".to_string();
        let saved = update.save(&pool).await.expect("save must succeed");

        assert_eq!(saved.id, saved_1.id, "the returned row must be the updated episode");
        assert_eq!(saved.title, "episode aaa111 (updated)");

        let count: i64 = query("SELECT count(*) FROM episodes WHERE title = $1")
            .bind("episode aaa111 (updated)")
            .map(|row: SqliteRow| row.get::<i64, _>(0))
            .fetch_one(&pool)
            .await
            .expect("count updated rows");
        assert_eq!(count, 1, "exactly one row must hold the new value");

        let untouched: i64 = query("SELECT count(*) FROM episodes WHERE yt_id = $1 AND title = $2")
            .bind("bbb222")
            .bind("episode bbb222")
            .map(|row: SqliteRow| row.get::<i64, _>(0))
            .fetch_one(&pool)
            .await
            .expect("count untouched rows");
        assert_eq!(untouched, 1, "the other episode must be untouched");
    }

    #[tokio::test]
    async fn update_returns_progressed_updated_at() {
        let pool = memory_pool().await;
        let channel_id = insert_channel(&pool).await;
        let created = episode_struct(channel_id, "ccc333");
        let saved = Episode::create(&pool, &created).await.expect("create");

        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        let mut update = saved.clone();
        update.title = "changed".to_string();
        let saved = update.save(&pool).await.expect("save");
        assert!(
            saved.updated_at >= saved.created_at,
            "updated_at must be refreshed on update"
        );
    }

    #[tokio::test]
    async fn remove_deletes_playlist_item_and_reindexes() {
        let pool = memory_pool().await;
        let channel_id = insert_channel(&pool).await;
        let first = Episode::create(&pool, &episode_struct(channel_id, "rem111"))
            .await
            .expect("create first");
        let middle = Episode::create(&pool, &episode_struct(channel_id, "rem222"))
            .await
            .expect("create middle");
        let last = Episode::create(&pool, &episode_struct(channel_id, "rem333"))
            .await
            .expect("create last");
        for episode in [&first, &middle, &last] {
            PlaylistItem::add(&pool, episode.id).await.expect("add playlist item");
        }

        Episode::remove(&pool, middle.id).await.expect("remove episode");

        let items = PlaylistItem::read_all(&pool).await.expect("read playlist");
        assert_eq!(items.iter().map(|item| item.episode_id).collect::<Vec<_>>(), vec![first.id, last.id]);
        assert_eq!(items.iter().map(|item| item.position).collect::<Vec<_>>(), vec![0, 1]);
    }

    #[tokio::test]
    async fn remove_non_playlist_episode_leaves_playlist_unchanged() {
        let pool = memory_pool().await;
        let channel_id = insert_channel(&pool).await;
        let kept = Episode::create(&pool, &episode_struct(channel_id, "keep11"))
            .await
            .expect("create kept");
        let removed = Episode::create(&pool, &episode_struct(channel_id, "gone22"))
            .await
            .expect("create removed");
        PlaylistItem::add(&pool, kept.id).await.expect("add kept item");

        Episode::remove(&pool, removed.id).await.expect("remove episode");

        let items = PlaylistItem::read_all(&pool).await.expect("read playlist");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].episode_id, kept.id);
        assert_eq!(items[0].position, 0);
    }

    #[tokio::test]
    async fn progress_update_stores_position_without_marking_listened() {
        let pool = memory_pool().await;
        let channel_id = insert_channel(&pool).await;
        let created = episode_struct(channel_id, "ddd444");
        let saved = Episode::create(&pool, &created).await.expect("create");
        assert!(!saved.listen);
        assert!(saved.listened_at.is_none());

        let updated = Episode::update_progress_by_yt_id(&pool, "ddd444", 1300, false)
            .await
            .expect("progress update must succeed");

        assert_eq!(updated.id, saved.id);
        assert_eq!(updated.yt_id, "ddd444");
        assert_eq!(updated.position_seconds, 1300);
        assert!(!updated.listen, "position-only update must not mark listened");
        assert!(
            updated.listened_at.is_none(),
            "position-only update must leave listened_at empty"
        );
    }

    #[tokio::test]
    async fn progress_update_with_listened_marks_the_episode() {
        let pool = memory_pool().await;
        let channel_id = insert_channel(&pool).await;
        let created = episode_struct(channel_id, "eee555");
        let _saved = Episode::create(&pool, &created).await.expect("create");

        let updated = Episode::update_progress_by_yt_id(&pool, "eee555", 3000, true)
            .await
            .expect("progress update must succeed");

        assert!(updated.listen, "listened flag must mark the episode played");
        assert!(
            updated.listened_at.is_some(),
            "listened_at must be set on completion"
        );
        assert_eq!(updated.position_seconds, 3000);
    }

    #[tokio::test]
    async fn progress_update_with_listened_false_clears_the_mark() {
        let pool = memory_pool().await;
        let channel_id = insert_channel(&pool).await;
        let created = episode_struct(channel_id, "fff666");
        let _saved = Episode::create(&pool, &created).await.expect("create");

        let marked = Episode::update_progress_by_yt_id(&pool, "fff666", 3000, true)
            .await
            .expect("mark must succeed");
        assert!(marked.listen);

        let cleared = Episode::update_progress_by_yt_id(&pool, "fff666", 0, false)
            .await
            .expect("unmark must succeed");
        assert!(!cleared.listen, "listened=false must clear the mark");
        assert!(
            cleared.listened_at.is_none(),
            "listened=false must clear listened_at"
        );
        assert_eq!(cleared.position_seconds, 0);
    }

    #[tokio::test]
    async fn progress_update_for_unknown_yt_id_errors() {
        let pool = memory_pool().await;
        let result = Episode::update_progress_by_yt_id(&pool, "unknown_yt_id_zzz", 10, false)
            .await;
        let err = result.expect_err("unknown episode must produce an error");
        assert_eq!(
            err.status_code(),
            StatusCode::NOT_FOUND,
            "a missing episode must be a 404, not a masked 500"
        );
    }

    #[tokio::test]
    async fn listened_at_does_not_drift_on_repeated_marked_saves() {
        let pool = memory_pool().await;
        let channel_id = insert_channel(&pool).await;
        let created = episode_struct(channel_id, "hhh888");
        let _saved = Episode::create(&pool, &created).await.expect("create");

        let first = Episode::update_progress_by_yt_id(&pool, "hhh888", 100, true)
            .await
            .expect("mark must succeed");
        let first_listened_at = first.listened_at;
        assert!(first_listened_at.is_some(), "must record the completion time");

        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        // A routine save of the same listened episode (frontend re-sends
        // listened=true while replaying) must keep the original timestamp.
        let again = Episode::update_progress_by_yt_id(&pool, "hhh888", 200, true)
            .await
            .expect("re-save must succeed");
        assert_eq!(
            again.listened_at.map(|t| t.timestamp()),
            first_listened_at.map(|t| t.timestamp()),
            "listened_at must not drift on re-saves of a listened episode"
        );
    }

    #[tokio::test]
    async fn read_progress_returns_stored_fields_by_yt_id() {
        let pool = memory_pool().await;
        let channel_id = insert_channel(&pool).await;
        let created = episode_struct(channel_id, "ggg777");
        let _saved = Episode::create(&pool, &created).await.expect("create");

        Episode::update_progress_by_yt_id(&pool, "ggg777", 899, true)
            .await
            .expect("mark must succeed");

        let progress = Episode::read_progress_by_yt_id(&pool, "ggg777")
            .await
            .expect("read must succeed");
        assert_eq!(progress.yt_id, "ggg777");
        assert_eq!(progress.position_seconds, 899);
        assert!(progress.listen);
        assert!(progress.listened_at.is_some());

        let missing = Episode::read_progress_by_yt_id(&pool, "missing_yt_zzz").await;
        assert!(missing.is_err(), "unknown episode must produce an error");
    }

    #[tokio::test]
    async fn created_episodes_default_to_not_favorite_and_serialize_the_flag() {
        let pool = memory_pool().await;
        let channel_id = insert_channel(&pool).await;
        let created = episode_struct(channel_id, "fav0001");
        let saved = Episode::create(&pool, &created).await.expect("create");
        assert!(!saved.favorite, "new episodes must default to not favorite");
        // The payload contract (episode-favorites): `favorite` is part of the
        // serialized episode so the frontend renders the star without a second
        // lookup.
        let json = serde_json::to_value(&saved).expect("serialize");
        assert_eq!(json["favorite"], false);
        assert_eq!(json["yt_id"], "fav0001");
    }

    #[tokio::test]
    async fn set_favorite_by_yt_id_toggles_and_404s_on_missing() {
        let pool = memory_pool().await;
        let channel_id = insert_channel(&pool).await;
        let created = episode_struct(channel_id, "fav0002");
        Episode::create(&pool, &created).await.expect("create");

        let marked = Episode::set_favorite_by_yt_id(&pool, "fav0002", true)
            .await
            .expect("mark must succeed");
        assert!(marked.favorite, "flag must be true after marking");

        let read = Episode::read(&pool, marked.id).await.expect("read back");
        assert!(read.favorite, "flag must persist in the row");

        let unmarked = Episode::set_favorite_by_yt_id(&pool, "fav0002", false)
            .await
            .expect("unmark must succeed");
        assert!(!unmarked.favorite, "flag must be false after unmarking");

        let missing = Episode::set_favorite_by_yt_id(&pool, "missing_fav_zzz", true).await;
        assert!(missing.is_err(), "unknown episode must 404");
    }
}


