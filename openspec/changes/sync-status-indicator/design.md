## Context

The app syncs each channel by fetching its latest videos and downloading new episodes. Two paths trigger a sync:

- Background worker (`src/utils/worker.rs::do_the_work`) iterates all channels periodically via `update_channel`.
- Manual per-channel refresh (`src/handlers/channels.rs::update_episodes`, `POST /channels/{channel}/update/`) spawns a background task calling the same `update_channel` (aliased as `refresh_channel`).

Neither path currently persists the outcome. `update_channel` returns `Result<(), Error>`, but `do_the_work` only logs failures and the manual handler returns the channel immediately (async, before sync finishes). Channel cards (`frontend/src/components/ChannelCard.vue`) render a top-right age badge; the top-left corner is free.

## Goals / Non-Goals

**Goals:**
- Persist per-channel last-sync outcome (timestamp + success/failure) in SQLite.
- Cover both sync paths (worker + manual).
- Expose the status in the channel API payload.
- Render a green/red dot in the top-left corner of each channel card.
- Backfill/neutral handling for channels never synced.

**Non-Goals:**
- No status history / audit trail of past syncs (only the latest outcome is kept).
- No automatic retry or alerting.
- No change to the async nature of the manual refresh endpoint (it stays fire-and-forget).
- No display of the error message in the UI (dot only), though the error may be stored for diagnostics.

## Decisions

**Decision: Store two nullable columns on `channels`.**
Add `last_sync_at DATETIME` (nullable) and `last_sync_ok BOOLEAN` (nullable) via a new migration. `last_sync_ok = NULL` means never synced; `TRUE`/`FALSE` mean success/failure. Rationale: minimal schema, no extra table, cheap to read in the existing `read_all`/`read` queries. Alternative considered: a separate `channel_syncs` history table — rejected, overkill for a single latest-outcome indicator.

**Decision: Record the outcome inside `update_channel`.**
`update_channel` already centralizes the per-channel sync and returns `Result`. Wrap its body so it writes the success/failure to the DB before returning. Because both `do_the_work` and the manual handler call `update_channel`, both paths are covered in one place. The manual handler's spawned task already calls `refresh_channel` and logs errors, so no handler change is needed for recording. Add a small helper (e.g. `Channel::set_sync_status(pool, id, ok)`) that sets `last_sync_at = now` and `last_sync_ok`.

**Decision: On failure, also store the error message (optional).**
Add a nullable `last_sync_error TEXT` column so the cause is available for future diagnostics/debugging. Optional but cheap. The UI does not render it in this change.

**Decision: Serialize the new fields on `Channel`.**
Add `last_sync_at: Option<DateTime<Utc>>` and `last_sync_ok: Option<bool>` to the `Channel` struct and populate them in `from_row`. The existing `SELECT *` / `read_all` queries pick up the new columns automatically. Frontend `Channel` type mirrors them as nullable.

**Decision: Green/red dot in the top-left corner.**
`ChannelCard.vue` already uses `absolute right-4 top-4` for the age badge. Add an `absolute left-4 top-4` dot: `bg-emerald-500` (success) / `bg-error` or `bg-red-500` (failure), sized ~`h-2.5 w-2.5` with a subtle ring/shadow to match the glass-card style. Render nothing when `last_sync_ok == null`. The indicator is a non-interactive `<span>`, placed so it does not overlap the age badge.

**Decision: Data flow for the manual refresh indicator.**
The manual refresh endpoint stays async and returns the channel before sync finishes, so the card cannot immediately flip on the in-progress refresh. The indicator updates on the next channel-list fetch (page load), which already re-pulls `read_all`. No polling is added in this change.

## Risks / Trade-offs

- [Async manual refresh means the dot lags until next load] → Acceptable; the indicator reflects the last *recorded* sync. Users see the result on next visit/refresh.
- [New DB columns require a migration on existing installs] → Standard forward migration; nullable columns are additive and non-breaking. Rollback: a down migration dropping the columns.
- [Recording status inside `update_channel` adds a DB write on every sync] → Trivial cost (single UPDATE per channel per sync cycle).
- [Red dot may look alarming for transient 429s] → Intended: user asked any non-expected error be red. No throttling added (non-goal).
