## Context

Episodes persist in SQLite (`migrations/20240316092627_episodes.up.sql`) with an existing `listen BOOLEAN NOT NULL DEFAULT FALSE` column that is never updated after creation (the worker always inserts `listen=false`). The backend is Actix-web + sqlx; models live in `src/models/` (`Episode` in `episode.rs`), handlers in `src/handlers/` (`episodes.rs` has `read_with_pagination` and `read_all`), all authenticated routes are scoped under `RequireSession` in `src/handlers/mod.rs`, and responses use the `CResponse` envelope.

The frontend player (`frontend/src/stores/player.ts`) drives the shared `<audio>` element and already listens to `timeupdate`, `pause`, `ended`, etc. The API client (`frontend/src/lib/api/client.ts`) sends cookies (`credentials: 'include'`).

## Goals / Non-Goals

**Goals:**
- Persist per-episode playback position server-side, per the shared SQLite DB (cross-device for the same login).
- Persist a "listened/played" mark per episode, set on completion.
- Resume automatically when replaying an episode with a meaningful saved position; offer "start over".
- Show the played mark and a resume hint on episode cards.

**Non-Goals:**
- Real-time multi-device sync, per-user position isolation (single-admin deployment).
- Playback analytics, time-series, or client-side position as source of truth.
- Changing the History screen semantics (it remains the ordered list of downloaded episodes).

## Decisions

### Decision 1: Two new columns, `listen` becomes the played mark

```sql
ALTER TABLE episodes ADD COLUMN position_seconds INTEGER NOT NULL DEFAULT 0;
ALTER TABLE episodes ADD COLUMN listened_at DATETIME;
```

`listen` (already present) is the boolean "played" mark; `listened_at` records when. `position_seconds` defaults to 0 so existing rows resume from the start.

**Why**: matches the existing `add_sync_status` migration pattern (sqlx ALTERs); reuses `listen` instead of adding a redundant flag column.

### Decision 2: Single endpoint `PUT /api/1.0/episodes/{yt_id}/progress/`

The episode is addressed by its public id (`yt_id`) — the same identity the media URLs use (`/media/{channel_slug}/{yt_id}.mp3`) — so progress is associated with the individual episode regardless of whether it was played alone or from a playlist/queue. Body: `{ "position_seconds": number, "listened": boolean }`. The handler resolves the row by `yt_id`, calls a `modify`-style model method on `Episode` (`update_progress` on the resolved row id), and answers `204 No Content` on success — the frontend save is fire-and-forget, so the status code is enough and no response body is emitted; a missing episode is answered with the `404` error envelope. Implemented as a small handler in `src/handlers/episodes.rs` registered in `src/handlers/mod.rs`.

**Why**: one round-trip covers save-position and mark-played; the client decides when each applies (`listened` only true on completion).

### Decision 3: Throttled client-side saving

The store saves position:
- at most once every 10s during playback (timestamp gate in the `timeupdate` handler),
- on `pause`, on `stop`, and on `ended`,
- on `pagehide`/`visibilitychange` (hidden) to catch tab closes.

Saves are fire-and-forget `api.updateEpisodeProgress(id, { position_seconds, listened })`, suppressed while `position_seconds` did not move. Save failure is logged, not surfaced (non-blocking updates pattern).

**Why**: 10s cadence keeps writes low on a single-user SQLite deployment; finalizing on pause/ended/unload avoids losing the last-seen position.

### Decision 4: Resume policy and thresholds

On `play(episode)`, the flow is: load the source, **wait for `loadedmetadata`** (so `duration` is known), call `el.play()`, and apply the resume **on the `playing` event** — the moment the element is genuinely streaming — where browsers reliably honor the seek:
- if `position_seconds > 30` and `< duration * 0.95` → `seek(position)` (resume);
- else → play from 0.

`play(episode, { fromStart: true })` opts out of resume ("start over" affordance), stops any in-flight resume retry loop, and immediately persists `position_seconds = 0`; it also resets the live playhead to zero so replaying the already-loaded current episode truly restarts.

**Why**: 30s skips accidental replay-induced seeks; 95% treats near-complete episodes as finished (ad-heavy endings). The seek is applied via a **resume retry loop**: playback starts and, while a resume is pending, the store re-issues `currentTime = target` on every `timeupdate` (plus a 500ms interval) until the playhead reaches the target or a 20s deadline passes. This is robust against browsers that, with a metadata-only `preload`, limit `seekable` to the buffered prefix and **silently clamp a single seek beyond it** (observed in both Firefox and Chrome: `seekable: "0-16.32"` while `duration` is correct); as playback continues the buffer — and thus `seekable` — grows until the seek lands, without forcing a full download (bandwidth-friendly). A target at or beyond 95% of the duration is treated as finished and playback restarts from zero. The same resume decision is shared by every way of landing on an episode — `play()`, the restored-queue restart (`togglePlay()`), auto-advance (`advance()`), next (`skipNext()`) and the dual previous control (`playPrevious()`), which now truly "continue" an episode from its saved start time; a fresh episode or one without a saved position starts at zero. (A departure note: `play()`/`togglePlay()` push nothing onto the playback history; only advancing/skipping does, which is what the previous control walks back.) While a resume is pending the **resume target** stays the authoritative position: premature writes (a playhead still clamped to the buffered prefix) are persisted as the target instead of the raw playhead, so pausing or closing the tab inside the retry window cannot lose the final position.

Before deciding, the resume reads the per-id progress map. That map is seeded from the episode **list** payloads (the episode endpoints already serialize `position_seconds`/`listen`/`listened_at`), so the normally played episodes resume without any per-episode request. Only episodes unknown to the session (e.g. a restored-queue entry not present in any fetched list) are looked up individually with `GET /api/1.0/episodes/{yt_id}/progress/`. The same applies when a *restored* queue episode is played again through `togglePlay()` (e.g. right after a reload, where the localStorage copy predates the saved progress), and the resume is applied after playback starts in both paths. `fromStart` skips all of this.

### Decision 5: Completion marks listened

In `onEnded` (before advancing per step 1): save `{ position_seconds: duration, listened: true }`, and optimistically update the in-memory episode + the card should it be visible.

The same mark path is reached from the step-2 long-press next control (`skipNext({ markCurrent: true })`), which stores `position_seconds` as the episode's duration exactly like completion. All marking goes through one shared `markListened()` helper in the store. A saved write never *regresses* a listened episode's position below its recorded value, so the departing-episode flush right after a long-press skip cannot overwrite the completion position with the live playhead. `markListened` falls back to the parsed `duration` string (`H:MM:SS`) when the media element has no usable duration, so a completed episode never records a zero position. Server-side, `listened_at` is only set on the false→true transition and otherwise kept (`UPDATE ... CASE WHEN listened THEN COALESCE(listened_at, $now) ELSE NULL`): re-saving an already-listened episode while replaying it never drifts the completion timestamp. The endpoint accepts `listened: false` as well — clearing the mark and `listened_at` — which the step-4 unmark flow uses with `position_seconds: 0`.

**Why**: completion is the agreed definition of "reproducido" (≥95% also counts via the resume threshold).

### Decision 6: Episode serialization and card UI

`Episode` (both `from_row` and `from_row_with_channel`) gains `position_seconds: i64` and `listened_at: Option<DateTime<Utc>>`; GET episode responses carry them automatically. `EpisodeCard` renders:
- played mark: check badge + "Escuchado" label when `listen` is true;
- resume hint: "Continuar en MM:SS" (formatted from `position_seconds`) when `listen` is false and position is above the 30s threshold;
- a "Start over" affordance when the card's episode is currently loaded with an active resume position.

**Why**: cards are the shared surface across both views; the History screen list itself stays untouched.

### Decision 7: Keyboard seek ±15 seconds

A window-level `keydown` listener (registered once in the player store; the app is a single-screen SPA so the store lives for the whole session) handles `ArrowRight`/`ArrowLeft`:

```
onKeydown(e):
  if !document.hasFocus()                        → ignore (no forced hijack)
  if episode not loaded                          → ignore
  if e.target is input/textarea/select/contenteditable → return (let it work)
  if e.target closest('[role=slider]')           → return (scrubber owns arrows)
  if ArrowRight → seekRelative(+15)
  if ArrowLeft  → seekRelative(-15)
```

`seekRelative(delta)` computes `clamp(currentTime + delta, 0, duration)` and calls `seek()`. The resulting position is persisted by the existing step-3 plumbing (throttled saves while playing, final save on pause/stop/ended), identical to scrubber seeks — no extra write path.

**Why**: ±15s is the standard web-player convention. The `document.hasFocus()` gate implements "only when the frontend is in focus", and the editable/slider guards prevent breaking text navigation in the search inputs and the scrubber.

**Alternative considered**: handling keys only when focus is inside the player bar or a card. Rejected by the "frontend in focus" requirement: the keys should work anywhere on the page without requiring a specific focus target.

## Risks / Trade-offs

- **[Risk] Frequent writes to SQLite.** Throttle to 10s + event-driven saves bounds writes to ~1 per 10s per active listener. → Acceptable for single-user; WAL + busy timeout already configured.
- **[Risk] `listened_at` drift / clock.** Single server clock; fine.
- **[Trade-off] Position is per-user in practice, but stored per-episode.** Single-admin reality makes per-episode storage the simplest correct answer; a `user_id` split is a later change if roles land.
- **[Risk] Resume races with rapid replay.** `fromStart` clears position immediately; resume applies only once per play via a flag consumed after seek.

## Migration Plan

1. Migration: add the two columns.
2. Backend: extend `Episode` model + `update_progress` method + PUT handler + route registration.
3. Frontend: API client method; store save/resume/completion logic; card badge/resume UI; i18n strings (en/es).
4. Tests: Rust model/handler tests (existing `episode_update_tests` pattern); Vitest for save throttling, resume thresholds, completion mark.
5. `cargo test`, `pnpm test`, `pnpm build`, manual verification per `tasks.md`.

**Rollback**: revert migration + code; dropping columns via `down.sql` (remove column requires table rebuild in SQLite — `down.sql` recreates the table).

## Open Questions

None.