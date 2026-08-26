use super::{Error, Episode};
use actix_web::http::StatusCode;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{
    query,
    sqlite::{SqlitePool, SqliteRow},
    Row,
};
use std::collections::HashSet;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistItem {
    pub id: i64,
    pub episode_id: i64,
    pub position: i64,
    pub added_at: DateTime<Utc>,
}

 // True when the sqlx error is a SQLite unique-constraint violation. On this
// table the only unique constraint is `UNIQUE(episode_id)`, so a violation of
// the add can only mean the episode is already in the playlist.
fn is_unique_violation(e: &sqlx::Error) -> bool {
    e.as_database_error()
        .map(|dbe| dbe.is_unique_violation())
        .unwrap_or(false)
}

impl PlaylistItem {
    fn from_row(row: SqliteRow) -> Self {
        info!("from_row");
        Self {
            id: row.get("id"),
            episode_id: row.get("episode_id"),
            position: row.get("position"),
            added_at: row.get("added_at"),
        }
    }

    // Plain playlist rows in stored order. The API read uses
    // `read_episodes_with_channels` for the joined payload; kept following the
    // `#[allow(dead_code)]` precedent for optional model helpers.
    #[allow(dead_code)]
    pub async fn read_all(pool: &SqlitePool) -> Result<Vec<Self>, Error> {
        info!("read_all");
        let sql = "SELECT * FROM playlist_items ORDER BY position ASC";
        query(sql)
            .map(Self::from_row)
            .fetch_all(pool)
            .await
            .map_err(|e| e.into())
    }

    /// The playlist's episodes in stored order, joined with channel slug/title
    /// so cards render channel links without extra requests. The INNER JOIN
    /// drops rows whose episode no longer exists (they stay in the table but
    /// are invisible; `reorder` cleans them up).
    pub async fn read_episodes_with_channels(pool: &SqlitePool) -> Result<Vec<Episode>, Error> {
        info!("read_episodes_with_channels");
        let sql = "SELECT e.*, COALESCE(c.slug, '') AS channel_slug, \
                   COALESCE(c.title, '') AS channel_title \
                   FROM playlist_items p \
                   INNER JOIN episodes e ON e.id = p.episode_id \
                   LEFT JOIN channels c ON c.id = e.channel_id \
                   ORDER BY p.position ASC";
        query(sql)
            .map(Episode::from_row_with_channel)
            .fetch_all(pool)
            .await
            .map_err(|e| e.into())
    }

    /// Appends an episode at `max(position)+1`. Adding an episode already in
    /// the playlist trips the `UNIQUE(episode_id)` constraint and surfaces as a
    /// 409 conflict leaving the list unchanged. The caller (handler) verifies
    /// the episode exists before calling.
    pub async fn add(pool: &SqlitePool, episode_id: i64) -> Result<Self, Error> {
        info!("add");
        let position = query("SELECT COALESCE(MAX(position), -1) + 1 FROM playlist_items")
            .map(|row: SqliteRow| row.get::<i64, _>(0))
            .fetch_one(pool)
            .await?;
        let sql = "INSERT INTO playlist_items (episode_id, position, added_at) \
                   VALUES ($1, $2, $3) RETURNING *;";
        match query(sql)
            .bind(episode_id)
            .bind(position)
            .bind(Utc::now())
            .map(Self::from_row)
            .fetch_one(pool)
            .await
        {
            Ok(item) => Ok(item),
            Err(e) if is_unique_violation(&e) => Err(Error::new_with_status_code(
                "episode already in playlist",
                StatusCode::CONFLICT,
            )),
            Err(e) => Err(e.into()),
        }
    }

    /// Deletes the episode from the playlist (404 when absent, per the pending
    /// semantics the frontend may fire removals for already-gone episodes) and
    /// reindexes the remaining positions contiguously. Returns the removed row.
    pub async fn remove(pool: &SqlitePool, episode_id: i64) -> Result<Self, Error> {
        info!("remove");
        let mut tx = pool.begin().await?;
        let item = query("DELETE FROM playlist_items WHERE episode_id = $1 RETURNING *;")
            .bind(episode_id)
            .map(Self::from_row)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or_else(|| {
                Error::new_with_status_code("episode not in playlist", StatusCode::NOT_FOUND)
            })?;
        Self::reindex(&mut *tx).await?;
        tx.commit().await?;
        Ok(item)
    }

    // Rewrites stored rows so positions become 0..n in `position` order.
    // Shared by `remove` and `reorder` so positions never drift.
    async fn reindex(conn: &mut sqlx::SqliteConnection) -> Result<(), Error> {
        let ids = query("SELECT id FROM playlist_items ORDER BY position ASC")
            .map(|row: SqliteRow| row.get::<i64, _>("id"))
            .fetch_all(&mut *conn)
            .await?;
        for (index, id) in ids.iter().enumerate() {
            query("UPDATE playlist_items SET position = $1 WHERE id = $2")
                .bind(index as i64)
                .bind(id)
                .execute(&mut *conn)
                .await?;
        }
        Ok(())
    }

    /// Rewrites positions in the given order. The submission must cover exactly
    /// the stored rows whose episodes still exist; rows referencing deleted
    /// episodes are dropped (they are invisible to the INNER JOIN read anyway).
    /// Any other mismatch (an episode added or removed server-side since the
    /// client's last read, a duplicate or unknown id) is a 409 so the frontend
    /// reloads instead of silently clobbering the list.
    pub async fn reorder(pool: &SqlitePool, episode_ids: &[i64]) -> Result<(), Error> {
        info!("reorder");
        let mut seen = HashSet::new();
        if episode_ids.iter().any(|id| !seen.insert(id)) {
            return Err(Error::new_with_status_code(
                "duplicate episode id in reorder",
                StatusCode::CONFLICT,
            ));
        }
        let stored: Vec<i64> = query("SELECT episode_id FROM playlist_items ORDER BY position ASC")
            .map(|row: SqliteRow| row.get::<i64, _>("episode_id"))
            .fetch_all(pool)
            .await?;
        let live: HashSet<i64> = query("SELECT id FROM episodes")
            .map(|row: SqliteRow| row.get::<i64, _>("id"))
            .fetch_all(pool)
            .await?
            .into_iter()
            .collect();
        let stored_live: HashSet<i64> =
            stored.iter().copied().filter(|id| live.contains(id)).collect();
        let submitted: HashSet<i64> = episode_ids.iter().copied().collect();
        if submitted.len() != episode_ids.len() || submitted != stored_live {
            return Err(Error::new_with_status_code(
                "playlist changed since last read; reload and retry",
                StatusCode::CONFLICT,
            ));
        }
        let mut tx = pool.begin().await?;
        for id in &stored {
            if !submitted.contains(id) {
                query("DELETE FROM playlist_items WHERE episode_id = $1")
                    .bind(id)
                    .execute(&mut *tx)
                    .await?;
            }
        }
        for (index, episode_id) in episode_ids.iter().enumerate() {
            query("UPDATE playlist_items SET position = $1 WHERE episode_id = $2")
                .bind(index as i64)
                .bind(episode_id)
                .execute(&mut *tx)
                .await?;
        }
        tx.commit().await?;
        Ok(())
    }
}

#[cfg(test)]
mod playlist_tests {
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

    async fn insert_channel(pool: &SqlitePool) -> i64 {
        let now = Utc::now();
        query(
            "INSERT INTO channels (url, title, slug, active, description, image, \
             first, max, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) RETURNING id",
        )
        .bind("https://example.com/pl")
        .bind("Playlist Test Channel")
        .bind("pl_test_channel")
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

    async fn insert_episode(pool: &SqlitePool, channel_id: i64, yt_id: &str) -> i64 {
        let now = Utc::now();
        query(
            "INSERT INTO episodes (channel_id, title, description, yt_id, webpage_url, \
             published_at, duration, image, listen, position_seconds, listened_at, \
             created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13) RETURNING id",
        )
        .bind(channel_id)
        .bind(format!("episode {yt_id}"))
        .bind("")
        .bind(yt_id)
        .bind(format!("https://youtu.be/{yt_id}"))
        .bind(now)
        .bind("00:10:00")
        .bind("")
        .bind(false)
        .bind(0i64)
        .bind(Option::<DateTime<Utc>>::None)
        .bind(now)
        .bind(now)
        .map(|row: SqliteRow| row.get::<i64, _>("id"))
        .fetch_one(pool)
        .await
        .expect("insert episode")
    }

    async fn add_all(pool: &SqlitePool, ids: &[i64]) {
        for id in ids {
            PlaylistItem::add(pool, *id).await.expect("add");
        }
    }

    #[tokio::test]
    async fn add_appends_episodes_in_position_order() {
        let pool = memory_pool().await;
        let channel = insert_channel(&pool).await;
        let ep1 = insert_episode(&pool, channel, "plaaa1").await;
        let ep2 = insert_episode(&pool, channel, "plbbb2").await;
        let ep3 = insert_episode(&pool, channel, "plccc3").await;
        add_all(&pool, &[ep1, ep2, ep3]).await;

        let items = PlaylistItem::read_all(&pool).await.expect("read_all");
        let ids: Vec<i64> = items.iter().map(|i| i.episode_id).collect();
        let positions: Vec<i64> = items.iter().map(|i| i.position).collect();
        assert_eq!(ids, vec![ep1, ep2, ep3], "append must land at the end");
        assert_eq!(positions, vec![0, 1, 2], "positions must be contiguous from 0");
    }

    #[tokio::test]
    async fn add_rejects_duplicates_with_conflict() {
        let pool = memory_pool().await;
        let channel = insert_channel(&pool).await;
        let ep = insert_episode(&pool, channel, "pldup1").await;
        PlaylistItem::add(&pool, ep).await.expect("first add");
        let err = PlaylistItem::add(&pool, ep)
            .await
            .expect_err("duplicate must fail");
        assert_eq!(err.status_code(), StatusCode::CONFLICT);
        let items = PlaylistItem::read_all(&pool).await.expect("read_all");
        assert_eq!(items.len(), 1, "duplicate add must leave the playlist unchanged");
    }

    #[tokio::test]
    async fn remove_reindexes_remaining_positions() {
        let pool = memory_pool().await;
        let channel = insert_channel(&pool).await;
        let ep1 = insert_episode(&pool, channel, "plrem1").await;
        let ep2 = insert_episode(&pool, channel, "plrem2").await;
        let ep3 = insert_episode(&pool, channel, "plrem3").await;
        add_all(&pool, &[ep1, ep2, ep3]).await;

        let removed = PlaylistItem::remove(&pool, ep2).await.expect("remove middle");
        assert_eq!(removed.episode_id, ep2);
        let items = PlaylistItem::read_all(&pool).await.expect("read_all");
        let ids: Vec<i64> = items.iter().map(|i| i.episode_id).collect();
        let positions: Vec<i64> = items.iter().map(|i| i.position).collect();
        assert_eq!(ids, vec![ep1, ep3], "relative order must be kept");
        assert_eq!(positions, vec![0, 1], "positions must be contiguous after removal");
    }

    #[tokio::test]
    async fn remove_unknown_episode_is_404() {
        let pool = memory_pool().await;
        let err = PlaylistItem::remove(&pool, 999)
            .await
            .expect_err("missing item must 404");
        assert_eq!(err.status_code(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn reorder_rewrites_positions_in_submitted_order() {
        let pool = memory_pool().await;
        let channel = insert_channel(&pool).await;
        let ep1 = insert_episode(&pool, channel, "plord1").await;
        let ep2 = insert_episode(&pool, channel, "plord2").await;
        let ep3 = insert_episode(&pool, channel, "plord3").await;
        add_all(&pool, &[ep1, ep2, ep3]).await;

        PlaylistItem::reorder(&pool, &[ep3, ep1, ep2])
            .await
            .expect("reorder full list");
        let items = PlaylistItem::read_all(&pool).await.expect("read_all");
        let ids: Vec<i64> = items.iter().map(|i| i.episode_id).collect();
        let positions: Vec<i64> = items.iter().map(|i| i.position).collect();
        assert_eq!(ids, vec![ep3, ep1, ep2], "stored order must match the submission");
        assert_eq!(positions, vec![0, 1, 2]);
    }

    #[tokio::test]
    async fn reorder_rejects_unknown_or_duplicate_ids() {
        let pool = memory_pool().await;
        let channel = insert_channel(&pool).await;
        let ep1 = insert_episode(&pool, channel, "plo1").await;
        let ep2 = insert_episode(&pool, channel, "plo2").await;
        add_all(&pool, &[ep1, ep2]).await;

        let err = PlaylistItem::reorder(&pool, &[ep1, ep2, 12345])
            .await
            .expect_err("unknown id must be rejected");
        assert_eq!(err.status_code(), StatusCode::CONFLICT);
        let err = PlaylistItem::reorder(&pool, &[ep1, ep1])
            .await
            .expect_err("duplicate id must be rejected");
        assert_eq!(err.status_code(), StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn reorder_omits_orphaned_episode_rows() {
        let pool = memory_pool().await;
        let channel = insert_channel(&pool).await;
        let ep1 = insert_episode(&pool, channel, "ploz1").await;
        let ep2 = insert_episode(&pool, channel, "ploz2").await;
        let ep3 = insert_episode(&pool, channel, "ploz3").await;
        add_all(&pool, &[ep1, ep2, ep3]).await;
        // Deleting the episode row leaves the playlist row orphaned (invisible
        // to the INNER JOIN read). Reordering the survivors must drop it.
        query("DELETE FROM episodes WHERE id = $1")
            .bind(ep2)
            .execute(&pool)
            .await
            .expect("delete episode row");

        PlaylistItem::reorder(&pool, &[ep3, ep1])
            .await
            .expect("reorder over an orphaned row");
        let items = PlaylistItem::read_all(&pool).await.expect("read_all");
        let ids: Vec<i64> = items.iter().map(|i| i.episode_id).collect();
        assert_eq!(ids, vec![ep3, ep1], "surviving ids keep the submitted order");
        assert_eq!(items.len(), 2, "the orphan row must be dropped");
    }

    #[tokio::test]
    async fn read_episodes_joins_channel_slug_and_title() {
        let pool = memory_pool().await;
        let channel = insert_channel(&pool).await;
        let ep1 = insert_episode(&pool, channel, "pljoi1").await;
        let ep2 = insert_episode(&pool, channel, "pljoi2").await;
        add_all(&pool, &[ep1, ep2]).await;

        let episodes = PlaylistItem::read_episodes_with_channels(&pool)
            .await
            .expect("join read");
        assert_eq!(episodes.len(), 2, "episodes must be returned in stored order");
        assert_eq!(
            episodes.iter().map(|e| e.id).collect::<Vec<_>>(),
            vec![ep1, ep2]
        );
        assert_eq!(episodes[0].channel_slug, "pl_test_channel");
        assert_eq!(episodes[0].channel_title, "Playlist Test Channel");
    }
}