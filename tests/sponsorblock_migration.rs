use sqlx::{migrate::Migrator, sqlite::SqlitePoolOptions, Row};
use std::path::Path;

#[tokio::test]
async fn sponsorblock_cache_migration_applies_and_rolls_back() {
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

    let columns = sqlx::query("PRAGMA table_info(sponsorblock_cache)")
        .fetch_all(&pool)
        .await
        .expect("read sponsorblock_cache columns");
    let names = columns
        .iter()
        .map(|row| row.get::<String, _>("name"))
        .collect::<Vec<_>>();
    assert_eq!(
        names,
        [
            "episode_id",
            "segments_json",
            "snapshot_hash",
            "checked_at",
            "processed_filename",
            "processed_duration",
            "last_error",
            "last_error_at",
            "processing_hash",
        ]
    );

    sqlx::raw_sql(include_str!(
        "../migrations/20260828000001_add_sponsorblock_cache.down.sql"
    ))
    .execute(&pool)
    .await
    .expect("roll back sponsorblock_cache migration");

    let table_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'sponsorblock_cache'",
    )
    .fetch_one(&pool)
    .await
    .expect("check sponsorblock_cache removal");
    assert_eq!(table_count, 0);
}

#[tokio::test]
async fn processing_hash_migration_backfills_existing_active_state() {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("memory pool");
    sqlx::raw_sql(
        "CREATE TABLE sponsorblock_cache (
            episode_id INTEGER PRIMARY KEY NOT NULL,
            segments_json TEXT NOT NULL,
            snapshot_hash TEXT NOT NULL,
            checked_at DATETIME NOT NULL,
            processed_filename TEXT,
            processed_duration REAL,
            last_error TEXT,
            last_error_at DATETIME
        );
        INSERT INTO sponsorblock_cache (
            episode_id, segments_json, snapshot_hash, checked_at,
            processed_filename, processed_duration
        ) VALUES (7, '[]', 'legacy-hash', '2026-08-29T00:00:00Z',
                  'video.sponsorblock.legacy.mp3', 42.0);",
    )
    .execute(&pool)
    .await
    .expect("create legacy state");

    sqlx::raw_sql(include_str!(
        "../migrations/20260829000001_add_sponsorblock_processing_hash.up.sql"
    ))
    .execute(&pool)
    .await
    .expect("apply processing hash migration");

    let row = sqlx::query(
        "SELECT snapshot_hash, processing_hash, processed_filename, processed_duration \
         FROM sponsorblock_cache WHERE episode_id = 7",
    )
    .fetch_one(&pool)
    .await
    .expect("read migrated state");
    assert_eq!(row.get::<String, _>("snapshot_hash"), "legacy-hash");
    assert_eq!(row.get::<String, _>("processing_hash"), "legacy-hash");
    assert_eq!(
        row.get::<String, _>("processed_filename"),
        "video.sponsorblock.legacy.mp3"
    );
    assert_eq!(row.get::<f64, _>("processed_duration"), 42.0);

    sqlx::raw_sql(include_str!(
        "../migrations/20260829000001_add_sponsorblock_processing_hash.down.sql"
    ))
    .execute(&pool)
    .await
    .expect("roll back processing hash migration");
    let columns = sqlx::query("PRAGMA table_info(sponsorblock_cache)")
        .fetch_all(&pool)
        .await
        .unwrap();
    assert!(!columns
        .iter()
        .any(|column| column.get::<String, _>("name") == "processing_hash"));
}

// Production connects with `foreign_keys(true)` (see src/main.rs). sqlx 0.9
// already turns SQLite foreign keys on by default per connection; this test
// locks in the behaviour the code depends on: deleting an episode cascades to
// its sponsorblock_cache row (no orphaned cache entries).
#[tokio::test]
async fn sponsorblock_cache_row_cascades_when_episode_is_deleted() {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("memory pool (sqlx enables foreign keys by default)");
    Migrator::new(Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations"))
        .await
        .expect("load migrations")
        .run(&pool)
        .await
        .expect("run migrations");

    // Guard the premise: the pool really enforces foreign keys.
    let fk: i64 = sqlx::query_scalar("PRAGMA foreign_keys")
        .fetch_one(&pool)
        .await
        .expect("read pragma");
    assert_eq!(fk, 1, "sqlx default pool must enforce foreign keys");

    let now = chrono::Utc::now();
    let channel_id: i64 = sqlx::query_scalar(
        "INSERT INTO channels (url, title, slug, active, description, image, first, max, created_at, updated_at) \
         VALUES ('https://example.com', 'FK Test', 'fk_test', TRUE, '', '', $1, 5, $1, $1) RETURNING id",
    )
    .bind(now)
    .fetch_one(&pool)
    .await
    .expect("insert channel");
    let episode_id: i64 = sqlx::query_scalar(
        "INSERT INTO episodes (channel_id, title, yt_id, webpage_url, published_at, duration, created_at, updated_at) \
         VALUES ($1, 'Ep', 'fk-ep', 'https://youtu.be/fk-ep', $2, '600', $2, $2) RETURNING id",
    )
    .bind(channel_id)
    .bind(now)
    .fetch_one(&pool)
    .await
    .expect("insert episode");
    sqlx::query(
        "INSERT INTO sponsorblock_cache (episode_id, segments_json, snapshot_hash, checked_at) \
         VALUES ($1, '[]', 'hash', $2)",
    )
    .bind(episode_id)
    .bind(now)
    .execute(&pool)
    .await
    .expect("insert sponsorblock cache row");

    // Sanity: the cache row exists before deletion.
    let before: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sponsorblock_cache WHERE episode_id = $1",
    )
    .bind(episode_id)
    .fetch_one(&pool)
    .await
    .expect("count before");
    assert_eq!(before, 1);

    sqlx::query("DELETE FROM episodes WHERE id = $1")
        .bind(episode_id)
        .execute(&pool)
        .await
        .expect("delete episode");

    // The ON DELETE CASCADE must have removed the cache row.
    let after: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sponsorblock_cache WHERE episode_id = $1",
    )
    .bind(episode_id)
    .fetch_one(&pool)
    .await
    .expect("count after");
    assert_eq!(after, 0, "sponsorblock_cache row must cascade on episode delete");
}
