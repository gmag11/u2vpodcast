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