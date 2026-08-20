## 1. Database

- [x] 1.1 Add migration adding nullable `last_sync_at DATETIME`, `last_sync_ok BOOLEAN`, and `last_sync_error TEXT` columns to the `channels` table (up + down)
- [x] 1.2 Verify migrations apply cleanly against a copy of `u2vpodcast.db`

## 2. Backend model

- [x] 2.1 Add `last_sync_at: Option<DateTime<Utc>>`, `last_sync_ok: Option<bool>`, and `last_sync_error: Option<String>` fields to the `Channel` struct
- [x] 2.2 Populate the new fields in `Channel::from_row`
- [x] 2.3 Add a helper (e.g. `Channel::set_sync_status(pool, id, ok, error)`) that updates `last_sync_at`, `last_sync_ok`, and `last_sync_error`

## 3. Sync recording

- [x] 3.1 Wrap `update_channel` in `src/utils/worker.rs` to record success (`last_sync_ok = true`) on `Ok` and failure (`last_sync_ok = false` + error message) on `Err`, while still returning the original result
- [x] 3.2 Confirm both sync paths (background `do_the_work` and manual `update_episodes` handler) flow through the recording logic

## 4. Frontend

- [x] 4.1 Add `last_sync_at`, `last_sync_ok`, and `last_sync_error` as nullable fields to the `Channel` type in `frontend/src/types.ts`
- [x] 4.2 Add a green/red indicator dot to the top-left corner of `ChannelCard.vue`: green (`last_sync_ok === true`), red (`last_sync_ok === false`), nothing when null; ensure it does not overlap the top-right age badge

## 5. Verification

- [x] 5.1 Build the Rust backend (`cargo build`) and run the frontend type check/lint
- [x] 5.2 Manually verify: a never-synced channel shows no dot, a successful sync shows green, and a failed sync (e.g. bad cookies / 429) shows red after the channel list is re-fetched
