use super::Error;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{
    query,
    sqlite::{SqlitePool, SqliteRow},
    Row,
};

fn default_category() -> String {
    "sponsor".to_string()
}

/// Categorized segment persisted in `sponsorblock_cache`. The category default
/// keeps snapshots written before categorized segments were introduced readable.
#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
pub struct SponsorBlockSegment {
    pub start: f64,
    pub end: f64,
    #[serde(default = "default_category")]
    pub category: String,
}

impl SponsorBlockSegment {
    pub fn new(start: f64, end: f64, category: &str) -> Self {
        Self {
            start,
            end,
            category: category.to_string(),
        }
    }
}

/// SponsorBlock shape exposed by episode APIs. Rejection is deployment policy,
/// so it is derived at serialization boundaries and is never persisted.
#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
pub struct EpisodeSponsorBlockSegment {
    pub start: f64,
    pub end: f64,
    pub category: String,
    pub rejected: bool,
}

impl SponsorBlockSegment {
    pub fn for_api(&self, rejected: bool) -> EpisodeSponsorBlockSegment {
        EpisodeSponsorBlockSegment {
            start: self.start,
            end: self.end,
            category: self.category.clone(),
            rejected,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SponsorBlockCache {
    pub episode_id: i64,
    pub segments: Vec<SponsorBlockSegment>,
    pub snapshot_hash: String,
    pub processing_hash: Option<String>,
    pub checked_at: DateTime<Utc>,
    pub processed_filename: Option<String>,
    pub processed_duration: Option<f64>,
    pub last_error: Option<String>,
    pub last_error_at: Option<DateTime<Utc>>,
}

impl SponsorBlockCache {
    fn from_row(row: SqliteRow) -> Self {
        let segments_json: String = row.get("segments_json");
        Self {
            episode_id: row.get("episode_id"),
            segments: serde_json::from_str(&segments_json).unwrap_or_default(),
            snapshot_hash: row.get("snapshot_hash"),
            processing_hash: row.get("processing_hash"),
            checked_at: row.get("checked_at"),
            processed_filename: row.get("processed_filename"),
            processed_duration: row.get("processed_duration"),
            last_error: row.get("last_error"),
            last_error_at: row.get("last_error_at"),
        }
    }

    pub async fn read(pool: &SqlitePool, episode_id: i64) -> Result<Option<Self>, Error> {
        query("SELECT * FROM sponsorblock_cache WHERE episode_id = $1")
            .bind(episode_id)
            .map(Self::from_row)
            .fetch_optional(pool)
            .await
            .map_err(Into::into)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn upsert_success(
        pool: &SqlitePool,
        episode_id: i64,
        segments: &[SponsorBlockSegment],
        snapshot_hash: &str,
        processing_hash: &str,
        processed_filename: Option<&str>,
        processed_duration: Option<f64>,
    ) -> Result<Self, Error> {
        let segments_json = serde_json::to_string(segments).map_err(|error| {
            Error::default(&format!("serialize SponsorBlock snapshot: {error}"))
        })?;
        let sql = "INSERT INTO sponsorblock_cache (
                       episode_id, segments_json, snapshot_hash, processing_hash, checked_at,
                       processed_filename, processed_duration, last_error, last_error_at
                   ) VALUES ($1, $2, $3, $4, $5, $6, $7, NULL, NULL)
                   ON CONFLICT(episode_id) DO UPDATE SET
                       segments_json = excluded.segments_json,
                       snapshot_hash = excluded.snapshot_hash,
                       processing_hash = excluded.processing_hash,
                       checked_at = excluded.checked_at,
                       processed_filename = excluded.processed_filename,
                       processed_duration = excluded.processed_duration,
                       last_error = NULL,
                       last_error_at = NULL
                   RETURNING *";
        query(sql)
            .bind(episode_id)
            .bind(segments_json)
            .bind(snapshot_hash)
            .bind(processing_hash)
            .bind(Utc::now())
            .bind(processed_filename)
            .bind(processed_duration)
            .map(Self::from_row)
            .fetch_one(pool)
            .await
            .map_err(Into::into)
    }

    pub async fn record_failure(
        pool: &SqlitePool,
        episode_id: i64,
        error: &str,
    ) -> Result<Option<Self>, Error> {
        query(
            "UPDATE sponsorblock_cache SET last_error = $1, last_error_at = $2 \
               WHERE episode_id = $3 RETURNING *",
        )
        .bind(error)
        .bind(Utc::now())
        .bind(episode_id)
        .map(Self::from_row)
        .fetch_optional(pool)
        .await
        .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Episode;
    use sqlx::{migrate::Migrator, sqlite::SqlitePoolOptions};
    use std::path::Path;

    async fn fixture() -> (SqlitePool, i64, i64) {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("memory pool");
        Migrator::new(Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations"))
            .await
            .expect("load migrations")
            .run(&pool)
            .await
            .expect("run migrations");
        query("PRAGMA foreign_keys = ON")
            .execute(&pool)
            .await
            .expect("enable foreign keys");
        let now = Utc::now();
        let channel_id: i64 = sqlx::query_scalar(
            "INSERT INTO channels (url, title, slug, active, description, image, first, max, created_at, updated_at) \
             VALUES ('https://example.com', 'Channel', 'channel', TRUE, '', '', $1, 5, $1, $1) RETURNING id",
        ).bind(now).fetch_one(&pool).await.expect("insert channel");
        let episode_id: i64 = sqlx::query_scalar(
            "INSERT INTO episodes (channel_id, title, yt_id, webpage_url, published_at, duration, created_at, updated_at) \
             VALUES ($1, 'Episode', 'video-id', 'https://example.com/video', $2, '600', $2, $2) RETURNING id",
        ).bind(channel_id).bind(now).fetch_one(&pool).await.expect("insert episode");
        (pool, channel_id, episode_id)
    }

    #[test]
    fn reads_legacy_and_categorized_segment_json() {
        let legacy: Vec<SponsorBlockSegment> =
            serde_json::from_str(r#"[{"start":10,"end":20}]"#).unwrap();
        assert_eq!(legacy, [SponsorBlockSegment::new(10.0, 20.0, "sponsor")]);
        let legacy_api = legacy[0].for_api(["sponsor"].contains(&legacy[0].category.as_str()));
        assert_eq!(legacy_api.category, "sponsor");
        assert!(legacy_api.rejected);
        let categorized: Vec<SponsorBlockSegment> =
            serde_json::from_str(r#"[{"start":10,"end":20,"category":"intro"}]"#).unwrap();
        assert_eq!(categorized, [SponsorBlockSegment::new(10.0, 20.0, "intro")]);
    }

    #[tokio::test]
    async fn distinguishes_unchecked_empty_and_non_empty_snapshots() {
        let (pool, channel_id, episode_id) = fixture().await;
        assert_eq!(
            SponsorBlockCache::read(&pool, episode_id).await.unwrap(),
            None
        );

        SponsorBlockCache::upsert_success(
            &pool,
            episode_id,
            &[],
            "empty-hash",
            "empty-processing",
            None,
            None,
        )
        .await
        .expect("store empty snapshot");
        let empty = SponsorBlockCache::read(&pool, episode_id)
            .await
            .unwrap()
            .unwrap();
        assert!(empty.segments.is_empty());
        assert_eq!(empty.processing_hash.as_deref(), Some("empty-processing"));

        let segments = [SponsorBlockSegment::new(10.0, 20.0, "intro")];
        SponsorBlockCache::upsert_success(
            &pool,
            episode_id,
            &segments,
            "non-empty-hash",
            "processing-hash",
            Some("video-id.sponsorblock.nonempty.mp3"),
            Some(590.0),
        )
        .await
        .expect("store non-empty snapshot");
        let mut episodes = Episode::read_episodes_for_channel(&pool, channel_id)
            .await
            .unwrap();
        episodes[0].apply_sponsorblock_config(true, &["intro".to_string()]);
        assert!(episodes[0].sponsorblock_enabled);
        assert_eq!(episodes[0].sponsorblock_segments[0].category, "intro");
        assert!(episodes[0].sponsorblock_segments[0].rejected);
        assert_eq!(
            episodes[0].sponsorblock_hash.as_deref(),
            Some("non-empty-hash")
        );
        let payload = serde_json::to_value(&episodes[0]).unwrap();
        assert_eq!(
            payload["sponsorblock_segments"][0],
            serde_json::json!({
                "start": 10.0,
                "end": 20.0,
                "category": "intro",
                "rejected": true
            })
        );
    }

    #[tokio::test]
    async fn cache_row_is_deleted_with_episode() {
        let (pool, _, episode_id) = fixture().await;
        SponsorBlockCache::upsert_success(
            &pool,
            episode_id,
            &[],
            "empty-hash",
            "empty-processing",
            None,
            None,
        )
        .await
        .expect("store snapshot");
        query("DELETE FROM episodes WHERE id = $1")
            .bind(episode_id)
            .execute(&pool)
            .await
            .expect("delete episode");
        assert_eq!(
            SponsorBlockCache::read(&pool, episode_id).await.unwrap(),
            None
        );
    }
}
