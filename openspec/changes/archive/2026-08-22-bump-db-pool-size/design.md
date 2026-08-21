## Context

Pool creation in `main.rs`:
```rust
SqlitePoolOptions::new().max_connections(2).connect(&db_url).await...
```
Two connections against: 2 HTTP worker threads (each can hold a slot while serving a request), the spawned background worker (`do_the_work` → per-channel tasks each clone the pool and run queries/download bookkeeping), and `actix_web::rt::spawn` refresh tasks from `create`/`update_episodes`. The `feed.xml` builders (`read_all_with_channels`, `read_episodes_for_channel`) run full joins; with 2 slots, a feed request + a refresh task leaves the API mostly queueing.

SQLite specifics: concurrent reads are safe; writes serialize on the DB file. Without WAL, a writer holds an exclusive lock and readers get `SQLITE_BUSY`. The current pool does not set WAL or a busy timeout, so even with more connections, contention can surface as `database is locked` errors under load. With WAL + `busy_timeout`, extra connections are mostly harmless.

## Goals / Non-Goals

**Goals:**
- Enough concurrent connections for workers + background + API traffic (default 5).
- WAL journal mode and a busy timeout configured at pool open.
- Configurable pool size with safe default (no unbounded growth).

**Non-Goals:**
- No query-level tuning or index changes (limits/offsets and ordering are separate specs).
- No async-write queue (the single-writer serialization stays in the DB).
- No migration of the DB file format beyond the journal-mode change (safe to enable WAL on an existing DB; `sqlx` PRAGMAs run at connect).

## Decisions

- **Default size: 5.** Justification: 2 workers could each hold 1 (2), the background worker + its per-channel clone may hold 1 (3), a fire-and-forget refresh may hold 1 (4), and 1 spare (5). A 6th waiter is acceptable rare; `max_connections` serves as a backpressure limit.
- **Config key:** optional `db_pool_max_connections: u32` in `Config` with serde default `5`. Loaded once; value clamped to `>= 1`. Pool uses `max_connections(config db pool value)`.
- **Journal/busy:** switch the pool to `SqliteConnectOptions`-based connect:
  ```rust
  SqliteConnectOptions::from_str(&db_url)?.journal_mode(SqliteJournalMode::Wal).busy_timeout(Duration::from_secs(5)).create_if_missing(true)
  ```
  then `SqlitePoolOptions::new().max_connections(n).connect_with(opts)`. This keeps `database_exists` behavior (or replace it with `create_if_missing(true)` and keep the migration run).
- **Rejected alternative — per-request pool sizing:** the pool is process-wide; per-request would fragment connections. Keep one shared pool.
- **Verification angle:** after enabling WAL, confirm a concurrent read + write does not raise `SQLITE_BUSY` under the manual load test.

## Risks / Trade-offs

- [More connections → more memory] → Trivial for SQLite (a few KB each).
- [WAL leaves `-wal`/`-shm` files next to the DB] → Expected SQLite behavior; backups must include or checkpoint the WAL (flag in migration notes).
- [Changing pool size interacts with the worker's sequential per-channel refresh] → Workers still run sequentially by design (`do_the_work` awaits each task); extra connections only help the HTTP layer, no behavior change to update scheduling.

## Migration Plan

1. Add `db_pool_max_connections` config key (default 5) and clamp.
2. Switch pool creation to `SqliteConnectOptions` with WAL + busy timeout + `create_if_missing`.
3. Confirm `Migrator` still runs against the pool and existing DBs open in WAL.
4. Load test: feed.xml + channel list + forced refresh concurrently — measure latency/flat `SQLITE_BUSY` absence; verify WAL files appear and a second run reuses the same DB.
5. Note WAL file handling in deployment docs/backup commands.

## Open Questions

None.