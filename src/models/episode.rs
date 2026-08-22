use super::Error;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{
    query,
    sqlite::{SqlitePool, SqliteRow},
    Row,
};
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
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

fn get_default_empty() -> String {
    "".to_string()
}

impl Episode {
    fn from_row(row: SqliteRow) -> Self {
        info!("from_row");
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
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

    fn from_row_with_channel(row: SqliteRow) -> Self {
        info!("from_row_with_channel");
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
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }

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
                   created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7,
                   $8, $9, $10, $11) RETURNING *;";
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
            .bind(episode.created_at)
            .bind(episode.updated_at)
            .map(Self::from_row)
            .fetch_one(pool)
            .await
            .map_err(|e| e.into())
    }

    pub async fn read_episodes_for_channel(pool: &SqlitePool, channel_id: i64) -> Result<Vec<Self>, Error>{
        info!("read_all");
        let sql = "SELECT * FROM episodes WHERE channel_id =$1 ORDER BY published_at DESC";
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
        let sql = "SELECT * FROM episodes ORDER BY published_at DESC";
        query(sql)
            .map(Self::from_row)
            .fetch_all(pool)
            .await
            .map_err(|e| e.into())
    }

    pub async fn read_all_with_channels(pool: &SqlitePool) -> Result<Vec<Self>, Error>{
        info!("read_all_with_channels");
        let sql = "SELECT e.*, COALESCE(c.slug, '') AS channel_slug, COALESCE(c.title, '') AS channel_title \
                   FROM episodes e LEFT JOIN channels c ON c.id = e.channel_id \
                   ORDER BY e.published_at DESC";
        query(sql)
            .map(Self::from_row_with_channel)
            .fetch_all(pool)
            .await
            .map_err(|e| e.into())
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
        let sql = "SELECT * FROM episodes
                   WHERE channel_id = $1 ORDER BY published_at DESC
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
                   duration = $7, image = $8, listen = $9, updated_at = $10
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
            .bind(updated_at)
            .map(Self::from_row)
            .fetch_one(pool)
            .await
            .map_err(|e| e.into())
    }

    pub async fn remove(pool: &SqlitePool, id: i64) -> Result<Episode, Error> {
        info!("remove");
        let sql = "DELETE from episodes WHERE id = $1 RETURNING * ;";
        query(sql)
            .bind(id)
            .map(Self::from_row)
            .fetch_one(pool)
            .await
            .map_err(|e| e.into())
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
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
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
}


