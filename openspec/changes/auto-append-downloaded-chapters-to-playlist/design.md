## Context

Channel syncs run in the background (`src/utils/worker.rs`): episodes are downloaded with `yt-dlp`, filtered against the retention `floor`, and persisted via `Episode::new`. Playlists are server-persisted in a `playlist_items` table with an explicit `position` column, `UNIQUE(episode_id)`, and strict append-at-end semantics already implemented by `PlaylistItem::add` (which rejects duplicates with a 409-style error). Today nothing connects the two: a downloaded chapter lands in the library but never in the playlist, so users manually add each episode after every sync.

## Goals / Non-Goals

**Goals:**
- Every successfully persisted download is appended to the end of the playlist automatically.
- Reuse the existing playlist append semantics (`PlaylistItem::add`): dedupe, end-position, no schema change.
- The append is best-effort: a failure is logged and never aborts the sync run.

**Non-Goals:**
- No UI or endpoint changes; no user-facing toggle.
- No re-append of already-playlisted episodes during resyncs (idempotency comes from the `UNIQUE(episode_id)` reject).
- No changes to playlist completion/removal flow (`playlist` spec is untouched).
- No version bump or release-specific work in this change.

## Decisions

**D1: Hook the append at the download-completion point in the worker.**
The worker is the single place where "an episode was actually downloaded and stored" is known (`Episode::new` succeeds, after the `floor` check). Appending right after the episode row is created guarantees the playlist only ever sees persisted episodes. It also guarantees ordering matches download order, which is what the user asked for ("al final de la playlist").
- Alternative rejected: appending at API/sync-request time — misses future non-worker download paths and would need the episode id resolved anyway.

**D2: Reuse `PlaylistItem::add` as-is.**
It already computes `MAX(position)+1`, rejects duplicates via the unique constraint, and uses the same code path the playlist API exposes — so the playlist read endpoint and the worker append can never disagree on semantics.
- Alternative rejected: a bespoke `INSERT OR IGNORE` in the worker — would bypass the shared dedupe/position logic and duplicate the "already in playlist" contract. Note: the unique-violation `Err` returned by `add` for duplicates is the *expected* outcome here, not a failure — the worker treats it as a no-op.

**D3: Best-effort with logging.**
The append call is `await`ed with `?`-free error handling: log `warn!`/`error!` on failure and continue the run. This satisfies the "append persists before the next download" and "failure tolerated" scenarios without coupling sync health to playlist state.

**D4: No config flag.**
The feature is always-on and server-side. A toggle would add config surface the user did not ask for.
- Alternative rejected (considered then dropped): `config.yml` opt-in flag — YAGNI for a single-instance app; can be added later if resync bloat ever becomes an issue.

## Risks / Trade-offs

- [Playlist grows with every sync, including re-downloads of edited/re-published episodes] → Dedupe handles same-episode repeats; if growth becomes a problem, a future opt-in flag (D4 alternative) bounds it.
- [Append DB error mid-sync silently "misses" an episode] → Logged at error level; playlist read still reflects previous state; a later sync of the same episode re-attempts via the same dedupe-safe path.
- [`add` returning `Err` on duplicates must not be confused with real failures] → Worker maps only the unique-violation error to no-op; all other errors are logged (covered by a task-level checklist).