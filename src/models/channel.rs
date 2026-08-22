use actix_web::http::StatusCode;
use serde::{
    Serialize,
    Deserialize
};
use std::fmt::{
    self,
    Display
};
use tracing::{
    info,
    debug,
    warn,
};
use chrono::{
    DateTime,
    Utc,
};
use regex::Regex;
use sqlx::{
    sqlite::{
        SqlitePool,
        SqliteRow
    },
    query,
    Row,
};

use super::{
    Error,
    Episode,
    YTInfo,
    cache_image,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Channel {
    pub id: i64,
    pub url: String,
    pub title: String,
    pub slug: String,
    pub active: bool,
    pub description: String,
    pub image: String,
    pub first: DateTime<Utc>,
    pub max: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_date: Option<DateTime<Utc>>,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub last_sync_ok: Option<bool>,
    pub last_sync_error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NewChannel {
    pub url: String,
    pub active: bool,
    pub first: DateTime<Utc>,
    pub max: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateChannel {
    pub id: i64,
    pub url: String,
    pub title: String,
    pub active: bool,
    pub first: DateTime<Utc>,
    pub max: i64,
}

impl Display for Channel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({} - {})", self.id, self.url)
    }
}

// True when the sqlx error is a SQLite unique-constraint violation specifically
// on the `channels.slug` column. We must NOT treat violations of other unique
// constraints (e.g. `channels.url` when creating the same channel twice) as a
// slug race, or the retry loop would spin unboundedly.
fn is_slug_unique_violation(e: &sqlx::Error) -> bool {
    e.as_database_error()
        .map(|dbe| dbe.is_unique_violation() && dbe.message().contains("channels.slug"))
        .unwrap_or(false)
}

fn slugify(title: &str) -> String {
    let folded = deunicode::deunicode(title).to_lowercase();
    let re = Regex::new(r"[^a-z0-9]+").unwrap();
    let slug = re.replace_all(&folded, "_").to_string();
    slug.trim_matches('_').to_string()
}

impl Channel{
    fn from_row(row: SqliteRow) -> Self{
        info!("from_row");
        Self{
            id: row.get("id"),
            url: row.get("url"),
            title: row.get("title"),
            slug: row.get("slug"),
            active: row.get("active"),
            description: row.get("description"),
            image: row.get("image"),
            first: row.get("first"),
            max: row.get("max"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            last_date: row.try_get("last_date").unwrap_or(None),
            last_sync_at: row.try_get("last_sync_at").unwrap_or(None),
            last_sync_ok: row.try_get("last_sync_ok").unwrap_or(None),
            last_sync_error: row.try_get("last_sync_error").unwrap_or(None),
        }
    }

    pub async fn new(pool: &SqlitePool, channel: NewChannel) -> Result<Self, Error>{
        info!("new");
        if channel.max < 1 {
            return Err(Error::new_with_status_code(
                "max must be >= 1",
                StatusCode::BAD_REQUEST,
            ));
        }
        let created_at = Utc::now();
        let updated_at = created_at;
        let ytinfo = match YTInfo::new(&channel.url).await{
            Ok(ytinfo) => ytinfo,
            Err(_) => YTInfo::default(),
        };
        let base_slug = slugify(&ytinfo.title);
        let sql = "INSERT INTO channels (url, title, slug, active, description,
                   image, first, max, created_at, updated_at)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) RETURNING *;";
        // Check-then-insert can lose a race: a second concurrent creation may
        // take the same slug between our uniqueness check and the INSERT. The
        // DB-level UNIQUE index turns that race into a unique-violation error,
        // and we retry with the next -N suffix instead of failing.
        let mut attempt = 0u32;
        let mut channel_row;
        loop {
            let slug = if attempt == 0 {
                Self::unique_slug(pool, &base_slug).await
            } else if base_slug.is_empty() {
                format!("channel-{}", attempt + 1)
            } else {
                format!("{}-{}", base_slug, attempt + 1)
            };
            match query(sql)
                .bind(&channel.url)
                .bind(&ytinfo.title)
                .bind(&slug)
                .bind(channel.active)
                .bind(&ytinfo.description)
                .bind(&ytinfo.image)
                .bind(channel.first)
                .bind(channel.max)
                .bind(created_at)
                .bind(updated_at)
                .map(Self::from_row)
                .fetch_one(pool)
                .await
            {
                Ok(row) => {
                    channel_row = row;
                    break;
                }
                Err(e) if is_slug_unique_violation(&e) => {
                    debug!("Slug `{slug}` raced; retrying with suffix {}", attempt + 1);
                    attempt += 1;
                    continue;
                }
                Err(e) => return Err(Error::default(&e.to_string())),
            }
        }
        if channel_row.slug.is_empty() {
            let fallback = format!("channel-{}", channel_row.id);
            let sql = "UPDATE channels SET slug = $1 WHERE id = $2 RETURNING *";
            channel_row = query(sql)
                .bind(&fallback)
                .bind(channel_row.id)
                .map(Self::from_row)
                .fetch_one(pool)
                .await
                .map_err(|e| Error::default(&e.to_string()))?;
        }
        // Populate the image cache now that the final slug is known: the API
        // `image` field must reference the local copy so rendering never opens
        // a YouTube connection for covers (channel-image-cache). A failed
        // download keeps the value inserted above (the remote URL) until the
        // next successful fetch (graceful degradation). A cache error is
        // logged, never allowed to fail the channel creation.
        if !channel_row.image.is_empty() {
            match cache_image(&channel_row.slug, &channel_row.image).await {
                Ok(Some(local)) if local != channel_row.image => {
                    let sql = "UPDATE channels SET image = $1 WHERE id = $2 RETURNING *";
                    channel_row = query(sql)
                        .bind(&local)
                        .bind(channel_row.id)
                        .map(Self::from_row)
                        .fetch_one(pool)
                        .await
                        .map_err(|e| Error::default(&e.to_string()))?;
                }
                Ok(_) => {}
                Err(e) => warn!(
                    "Cant cache image for new channel {}: {}",
                    channel_row.id, e
                ),
            }
        }
        Ok(channel_row)
    }

    async fn slug_exists(pool: &SqlitePool, slug: &str) -> bool {
        let sql = "SELECT count(*) FROM channels WHERE slug = $1";
        match query(sql)
            .bind(slug)
            .map(|row: SqliteRow| -> i64 { row.get(0) })
            .fetch_one(pool)
            .await
        {
            Ok(count) => count > 0,
            Err(_) => false,
        }
    }

    async fn unique_slug(pool: &SqlitePool, base: &str) -> String {
        if base.is_empty() {
            return String::new();
        }
        if !Self::slug_exists(pool, base).await {
            return base.to_string();
        }
        let mut n = 2;
        loop {
            let candidate = format!("{base}-{n}");
            if !Self::slug_exists(pool, &candidate).await {
                return candidate;
            }
            n += 1;
        }
    }

    pub async fn read_by_slug(pool: &SqlitePool, slug: &str) -> Result<Self, Error>{
        info!("read_by_slug");
        let sql = "SELECT * FROM channels WHERE slug = $1";
        query(sql)
            .bind(slug)
            .map(Self::from_row)
            .fetch_one(pool)
            .await
            .map_err(|e| Error::new_with_status_code(&e.to_string(), StatusCode::NOT_FOUND))
    }

    pub async fn read_by_id_or_slug(pool: &SqlitePool, key: &str) -> Result<Self, Error>{
        info!("read_by_id_or_slug");
        match key.parse::<i64>(){
            Ok(id) => Self::read(pool, id).await,
            Err(_) => Self::read_by_slug(pool, key).await,
        }
    }

    pub async fn count_by_slug(pool: &SqlitePool, slug: &str) -> Result<i64, Error>{
        let sql = "SELECT count(*) FROM channels WHERE slug = $1";
        query(sql)
            .bind(slug)
            .map(|row: SqliteRow| -> i64 { row.get(0) })
            .fetch_one(pool)
            .await
            .map_err(|e| e.into())
    }

    pub async fn read(pool: &SqlitePool, id: i64) -> Result<Self, Error>{
        info!("read");
        let sql = "SELECT * FROM channels WHERE id = $1";
        query(sql)
            .bind(id)
            .map(Self::from_row)
            .fetch_one(pool)
            .await
            .map_err(|e| Error::new_with_status_code(&e.to_string(), StatusCode::NOT_FOUND))
    }

    pub async fn read_all(pool: &SqlitePool) -> Result<Vec<Self>, Error>{
        info!("read_all");
        let sql = "SELECT c.*, e.last_date FROM channels c \
                   LEFT JOIN (SELECT channel_id, MAX(published_at) AS last_date \
                              FROM episodes GROUP BY channel_id) e \
                   ON e.channel_id = c.id \
                   ORDER BY e.last_date IS NULL, e.last_date DESC";
        query(sql)
            .map(Self::from_row)
            .fetch_all(pool)
            .await
            .map_err(|e| e.into())
    }

    // Channels the scheduled worker is allowed to process: only those with the
    // active flag set. Kept separate from `read_all` on purpose - the SPA
    // channel list and the slug migration both need *all* channels, inactive
    // included (otherwise a deactivated channel could not be re-enabled from
    // the UI and would never get its slug backfilled).
    pub async fn read_active(pool: &SqlitePool) -> Result<Vec<Self>, Error>{
        info!("read_active");
        let sql = "SELECT c.*, e.last_date FROM channels c \
                   LEFT JOIN (SELECT channel_id, MAX(published_at) AS last_date \
                              FROM episodes GROUP BY channel_id) e \
                   ON e.channel_id = c.id \
                   WHERE c.active = 1 \
                   ORDER BY e.last_date IS NULL, e.last_date DESC";
        query(sql)
            .map(Self::from_row)
            .fetch_all(pool)
            .await
            .map_err(|e| e.into())
    }

    pub async fn read_with_pagination(
        pool: &SqlitePool,
        page: i64,
        per_page: i64,
    ) -> Result<Vec<Channel>, Error> {
        tracing::debug!("Página: {page}. Páginas: {per_page}");
        // A malformed page (<= 0) must never yield a negative SQL OFFSET.
        let page = page.max(1);
        let offset = (page - 1) * per_page;
        // Paginate with the same ordering as `read_all` (most recent activity
        // first) so paginating the list does not silently reorder the UI
        // (fix-api-contract-mismatches / channels-list-ordering).
        let sql = "SELECT c.*, e.last_date FROM channels c \
                   LEFT JOIN (SELECT channel_id, MAX(published_at) AS last_date \
                              FROM episodes GROUP BY channel_id) e \
                   ON e.channel_id = c.id \
                   ORDER BY e.last_date IS NULL, e.last_date DESC \
                   LIMIT $1 OFFSET $2";
        query(sql)
            .bind(per_page)
            .bind(offset)
            .map(Self::from_row)
            .fetch_all(pool)
            .await
            .map_err(|e| e.into())
    }

    pub async fn update(pool: &SqlitePool, channel: &UpdateChannel) -> Result<Self, Error>{
        info!("update");
        debug!("{:?}", channel);
        if channel.max < 1 {
            return Err(Error::new_with_status_code(
                "max must be >= 1",
                StatusCode::BAD_REQUEST,
            ));
        }
        if channel.title.trim().is_empty() {
            return Err(Error::new_with_status_code(
                "Channel title cannot be empty",
                StatusCode::BAD_REQUEST,
            ));
        }
        let updated_at = Utc::now();
        // The slug stays immutable: renaming a channel must not change its slug
        // or audio directory (see channel-slugs spec).
        let sql = "UPDATE channels SET url = $1, active = $2, first = $3, max = $4,
                   title = $5, updated_at = $6 WHERE id = $7 RETURNING *";
        query(sql)
            .bind(&channel.url)
            .bind(channel.active)
            .bind(channel.first)
            .bind(channel.max)
            .bind(&channel.title)
            .bind(updated_at)
            .bind(channel.id)
            .map(Self::from_row)
            .fetch_one(pool)
            .await
            .map_err(|e| e.into())
    }

    pub async fn update_image(pool: &SqlitePool, id: i64, url: &str) -> Result<Self, Error>{
        info!("update_image");
        let channel = Self::read(pool, id).await?;
        Self::refresh_image_inner(pool, &channel, url).await
    }

    // Shared refresh logic for manual (`update_image`) and worker-triggered
    // (`refresh_cached_image`) refreshes: fetch fresh metadata, probe +
    // download the cover into the local cache, and point `channel.image` at
    // the local URL. A failed download keeps the previously stored image (the
    // field is never blanked); only the metadata fetch itself can fail here,
    // which the caller decides how to surface.
    async fn refresh_image_inner(
        pool: &SqlitePool,
        channel: &Channel,
        url: &str,
    ) -> Result<Self, Error>{
        let ytinfo = YTInfo::new(url).await?;
        let mut image = channel.image.clone();
        if !ytinfo.image.is_empty() {
            match cache_image(&channel.slug, &ytinfo.image).await {
                Ok(Some(local)) => image = local,
                Ok(None) => {}
                // Cache write failure: keep the previously stored image and
                // log; the refresh itself must not fail the operation
                // (channel-image-cache: "keep the old file, never blank it").
                Err(e) => warn!(
                    "Cant cache image for channel {}: {}",
                    channel.id, e
                ),
            }
        }
        let updated_at = Utc::now();
        let sql = "UPDATE channels SET image = $1, updated_at = $2 WHERE id = $3 RETURNING *";
        query(sql)
            .bind(&image)
            .bind(updated_at)
            .bind(channel.id)
            .map(Self::from_row)
            .fetch_one(pool)
            .await
            .map_err(|e| e.into())
    }

    // Worker-facing image refresh during a channel sync. Best-effort by
    // convention: the caller logs a failure instead of failing the whole sync,
    // and the previous stored image is kept (refresh_image_inner semantics).
    pub async fn refresh_cached_image(
        pool: &SqlitePool,
        channel: &Channel,
    ) -> Result<(), Error> {
        info!("refresh_cached_image");
        Self::refresh_image_inner(pool, channel, &channel.url)
            .await
            .map(|_| ())
    }

    pub async fn delete(pool: &SqlitePool, id: i64) -> Result<Self, Error>{
        info!("delete");
        let mut tx = pool.begin().await?;
        query("DELETE FROM episodes WHERE channel_id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;
        let sql = "DELETE FROM channels WHERE id = $1 RETURNING *";
        let channel = query(sql)
            .bind(id)
            .map(Self::from_row)
            .fetch_one(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(channel)
    }

    pub async fn migrate_slugs(pool: &SqlitePool, audio_folder: &str) -> Result<(), Error>{
        info!("migrate_slugs");
        let channels = Self::read_all(pool).await?;
        for channel in channels.as_slice(){
            if channel.slug.is_empty(){
                let base_slug = slugify(&channel.title);
                let mut slug = Self::unique_slug(pool, &base_slug).await;
                if slug.is_empty(){
                    slug = format!("channel-{}", channel.id);
                }
                let sql = "UPDATE channels SET slug = $1 WHERE id = $2";
                query(sql)
                    .bind(&slug)
                    .bind(channel.id)
                    .execute(pool)
                    .await
                    .map_err(|e| Error::default(&e.to_string()))?;
                info!("Backfilled slug {} for channel {}", &slug, channel.id);
            }
        }
        let channels = Self::read_all(pool).await?;
        for channel in channels.as_slice(){
            if channel.slug.is_empty(){
                continue;
            }
            let from = format!("{audio_folder}/{}", channel.id);
            let to = format!("{audio_folder}/{}", channel.slug);
            let from_exists = tokio::fs::metadata(&from).await.map(|_| true).unwrap_or(false);
            let to_exists = tokio::fs::metadata(&to).await.map(|_| true).unwrap_or(false);
            if from_exists && !to_exists {
                tokio::fs::rename(&from, &to)
                    .await
                    .map_err(|e| Error::default(&e.to_string()))?;
                info!("Renamed {from} -> {to}");
            }
        }
        Ok(())
    }

    #[allow(unused)]
    pub async fn number_of_channels(pool: &SqlitePool) -> i64 {
        let sql = "SELECT count(*) FROM channels";
        match query(sql)
            .map(|row: SqliteRow| -> i64 { row.get(0) })
            .fetch_one(pool)
            .await
        {
            Ok(value) => value,
            Err(e) => {
                tracing::info!("Error on exists {}", e);
                0
            }
        }
    }

    pub async fn number_of_episodes(&self, pool: &SqlitePool) -> i64 {
        Self::total(pool, self.id).await
    }

    pub async fn total(pool: &SqlitePool, channel_id: i64) -> i64 {

        let sql = "SELECT count(*) FROM episodes WHERE channel_id = $1";
        match query(sql)
            .bind(channel_id)
            .map(|row: SqliteRow| -> i64 { row.get(0) })
            .fetch_one(pool)
            .await
        {
            Ok(value) => value,
            Err(e) => {
                tracing::info!("Error on exists {}", e);
                0
            }
        }
    }

    pub async fn episode_exists(&self, pool: &SqlitePool, yt_id: &str) -> bool{
        Episode::exists(pool, self.id, yt_id).await
    }

    pub async fn get_max_date(&self, pool: &SqlitePool) -> DateTime<Utc> {
        let sql = "SELECT MAX(published_at) as last_date FROM episodes WHERE channel_id = $1";
        match query(sql).bind(self.id).fetch_one(pool).await {
            Ok(row) => row.get(0),
            Err(e) => {
                tracing::info!("Not last: {}", e);
                Utc::now()
            }
        }
    }

    pub async fn set_sync_status(
        pool: &SqlitePool,
        id: i64,
        ok: bool,
        error: Option<String>,
    ) -> Result<(), Error>{
        let sql = "UPDATE channels SET last_sync_at = $1, last_sync_ok = $2, \
                   last_sync_error = $3 WHERE id = $4";
        query(sql)
            .bind(Utc::now())
            .bind(ok)
            .bind(error)
            .bind(id)
            .execute(pool)
            .await
            .map_err(|e| Error::default(&e.to_string()))?;
        Ok(())
    }
}

#[cfg(test)]
mod channel_pagination_tests {
    use super::*;
    use sqlx::{
        migrate::Migrator,
        sqlite::SqlitePoolOptions,
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

    async fn insert_channel(pool: &SqlitePool, url: &str, title: &str) -> i64 {
        let now = Utc::now();
        query(
            "INSERT INTO channels (url, title, slug, active, description, image, \
             first, max, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) RETURNING id",
        )
        .bind(url)
        .bind(title)
        .bind(slugify(title))
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

    #[tokio::test]
    async fn pagination_pages_are_disjoint_and_bounded() {
        let pool = memory_pool().await;
        let mut ids = Vec::new();
        for i in 1..=3 {
            ids.push(insert_channel(&pool, &format!("https://example.com/c{i}"), &format!("Channel {i}")).await);
        }

        let page1 = Channel::read_with_pagination(&pool, 1, 2).await.expect("page 1");
        let page2 = Channel::read_with_pagination(&pool, 2, 2).await.expect("page 2");

        assert_eq!(page1.len(), 2);
        assert_eq!(page2.len(), 1);

        let ids1: std::collections::HashSet<i64> = page1.iter().map(|c| c.id).collect();
        let ids2: std::collections::HashSet<i64> = page2.iter().map(|c| c.id).collect();
        assert!(ids1.is_disjoint(&ids2), "pages must not overlap");
        assert_eq!(ids1.union(&ids2).cloned().collect::<std::collections::HashSet<_>>().len(), 3);
    }

    #[tokio::test]
    async fn zero_or_negative_page_is_clamped() {
        let pool = memory_pool().await;
        insert_channel(&pool, "https://example.com/c1", "Channel 1").await;
        insert_channel(&pool, "https://example.com/c2", "Channel 2").await;

        // page 0 and page -3 must behave as page 1, never a negative OFFSET.
        let p0 = Channel::read_with_pagination(&pool, 0, 2).await.expect("page 0");
        let pneg = Channel::read_with_pagination(&pool, -3, 2).await.expect("negative page");
        assert_eq!(p0.len(), 2);
        assert_eq!(pneg.len(), 2);
    }
}


