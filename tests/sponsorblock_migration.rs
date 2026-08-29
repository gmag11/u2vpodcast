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
