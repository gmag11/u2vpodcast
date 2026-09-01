use sqlx::{migrate::Migrator, sqlite::SqlitePoolOptions, Row};
use std::path::Path;

#[tokio::test]
async fn chapter_migration_applies_to_a_fresh_database() {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    Migrator::new(Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations"))
        .await
        .unwrap()
        .run(&pool)
        .await
        .unwrap();

    let columns = sqlx::query("PRAGMA table_info(episodes)")
        .fetch_all(&pool)
        .await
        .unwrap();
    let chapter_column = columns
        .iter()
        .find(|row| row.get::<String, _>("name") == "chapters_json")
        .expect("chapters_json column");
    assert_eq!(chapter_column.get::<String, _>("type"), "TEXT");
    assert_eq!(chapter_column.get::<i64, _>("notnull"), 0);
}

#[tokio::test]
async fn chapter_migration_preserves_existing_episode_rows() {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::query("CREATE TABLE episodes (id INTEGER PRIMARY KEY, title TEXT NOT NULL)")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO episodes (title) VALUES ('Existing')")
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query(include_str!(
        "../migrations/20260831000001_add_episode_chapters.up.sql"
    ))
    .execute(&pool)
    .await
    .unwrap();

    let row = sqlx::query("SELECT title, chapters_json FROM episodes")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(row.get::<String, _>("title"), "Existing");
    assert_eq!(row.get::<Option<String>, _>("chapters_json"), None);
}
