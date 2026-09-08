## 1. Configurable pool size

- [x] 1.1 Add optional `db_pool_max_connections: u32` to `Config` with serde default `5` and clamp `>= 1`
- [x] 1.2 Use it in `SqlitePoolOptions::new().max_connections(...)` in `src/main.rs`

## 2. WAL journal mode and busy timeout

- [x] 2.1 Switch pool creation to `SqliteConnectOptions` (from the same `db_url`) with `SqliteJournalMode::Wal`, a busy timeout (~5s), and `create_if_missing(true)`, then `connect_with`
- [x] 2.2 Keep the migration run against the pool working; reconcile the existing `database_exists`/`create_database` logic (replace or keep, whichever stays correct with `create_if_missing`)
- [x] 2.3 Confirm the DB opens in WAL (`PRAGMA journal_mode` = `wal`) after first run

## 3. Verification

- [x] 3.1 `cargo test` passes
- [x] 3.2 Concurrent load check: feed.xml + `/channels/` + a forced refresh in parallel — no `database is locked` / `SQLITE_BUSY`, latency flat
- [x] 3.3 Existing database opens and migrates cleanly after the journal-mode change
- [x] 3.4 Deployment docs: note `-wal`/`-shm` files in backup/restore instructions

## 4. Regression

- [x] 4.1 Re-run the bug-review reference (`docs/bug-review-2026-08-21.md`) checks around worker and API behavior; no regressions from the pool/WAL change