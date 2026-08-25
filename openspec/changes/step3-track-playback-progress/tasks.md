## 1. Migration

- [x] 1.1 Create `migrations/<timestamp>_add_playback_progress.up.sql`: `ALTER TABLE episodes ADD COLUMN position_seconds INTEGER NOT NULL DEFAULT 0;` and `ALTER TABLE episodes ADD COLUMN listened_at DATETIME;`
- [x] 1.2 Create the matching `.down.sql` (SQLite: recreate the `episodes` table without the two columns, copying existing rows).
- [x] 1.3 Confirm the migration runs (`cargo test` uses the memory-pool Migrator; also run the app once or `sqlx` migration check).

## 2. Backend model

- [x] 2.1 Extend `src/models/episode.rs`: add `pub position_seconds: i64` and `pub listened_at: Option<DateTime<Utc>>`; populate them in `from_row` and `from_row_with_channel`.
- [x] 2.2 Update `Episode::create`/`new` and the `episodes` INSERT to include `position_seconds` (0) and null `listened_at`.
- [x] 2.3 Add method `update_progress(pool, id, position_seconds, listened) -> Result<Episode>` updating position, `listen`, and `listened_at` (set to `Utc::now()` when `listened` true) and returning the refreshed row.
- [x] 2.4 Update the existing `update` SQL and fix all struct literals in `episode.rs` (including the test helper `episode_struct`) for the new fields.

## 3. Backend handler + route

- [x] 3.1 In `src/handlers/episodes.rs` add `#[put("/episodes/{yt_id}/progress/")]` handler parsing `{ position_seconds: i64, listened: bool }`, calling `update_progress_by_yt_id` (progress associated to the episode's public id), returning `CResponse::ok(session, episode)`; use `CResponse::ko` for missing episode.
- [x] 3.2 Register the route in `src/handlers/mod.rs` inside the `RequireSession` scope (next to `episodes::read_all`).

## 4. Frontend API client

- [x] 4.1 In `frontend/src/lib/api/client.ts` add `updateEpisodeProgress(ytId: string, body: { position_seconds: number; listened: boolean })` calling `PUT /api/1.0/episodes/${ytId}/progress/`.
- [x] 4.2 In `frontend/src/types.ts` add `position_seconds: number` and `listened_at: string | null` to `Episode`.

## 5. Player store save/resume/completion

- [x] 5.1 In `frontend/src/stores/player.ts` track last saved time; in the existing `timeupdate` handler save position at most every 10s via `updateEpisodeProgress` (skip when position did not change).
- [x] 5.2 Save on `pause`, on `stop`, and on `ended`; register a `pagehide`/`visibilitychange(hidden)` listener on the audio element or window to flush the final position.
- [x] 5.3 Resume: in `play()`, capture a `shouldResume` flag when `episode.position_seconds > 30`; on `loadedmetadata`, if the flag is set and `position < duration * 0.95`, `seek(position)` and clear the flag. The same path covers navigation back (step-2 dual previous), which reuses `play()`.
- [x] 5.4 Add opt-out: `play(episode, list?, { fromStart?: boolean })` — when `fromStart` is true skip resume and immediately persist `position_seconds = 0`.
- [x] 5.5 In `onEnded` (before advance from step 1/2): save `{ position_seconds: duration, listened: true }` and update the in-memory current episode (`listen=true`, `listened_at`), sending progress with `listened: true`. Extract a shared `markListened()` helper so the step-2 long-press skip (`skipNext({ markCurrent: true })`) uses the same path.
- [x] 5.6 Suppress resume for repeat-one replays (step 5) by reusing `fromStart`.
- [x] 5.7 Add `seekRelative(delta)` (clamp `currentTime + delta` to `[0, duration]` then `seek()`) and register the window `keydown` listener in the store: handle `ArrowRight`/`ArrowLeft` only when `document.hasFocus()`, an episode is loaded, and the event target is not an input/textarea/select/contenteditable or inside a `[role=slider]`.

## 6. Card badge and resume UI

- [x] 6.1 In `frontend/src/components/EpisodeCard.vue` render a played mark (check icon + "listened" label) when `props.episode.listen` is true.
- [x] 6.2 When `listen` is false and `position_seconds > 30`, render a "Continue at MM:SS" hint (format via the existing `formatter`/duration helpers).
- [x] 6.3 Add a "start over" affordance when the card's episode is the current one and it has an active resume position; wire it to `play(episode, list, { fromStart: true })`.
- [x] 6.4 Add the i18n strings in `frontend/src/i18n/locales/en.json` and `es.json` (listened, continue-at, start-over) and keep locale parity (run the parity test).

## 7. Backend tests

- [x] 7.1 Add tests in `src/models/episode.rs` (existing memory-pool pattern): progress update sets position only; progress update with `listened` sets `listen=true` + `listened_at`; progress update with `listened=false` clears both `listen` and `listened_at` while storing the position; unknown id errors.
- [x] 7.2 Add a handler test if the handler pattern supports it (otherwise cover via model tests and `cargo test`.

## 8. Frontend tests

- [x] 8.1 Extend `frontend/src/stores/player.test.ts`: position saved on pause/ended; throttled saves at most every 10s; resume seeks only when `position_seconds > 30` and `< duration*0.95`; `fromStart` clears position; completion sends `listened: true`.
- [x] 8.2 Add a card test asserting the played mark and resume hint render conditions.
- [x] 8.3 Add keyboard-seek tests (fake key events): `ArrowRight` seeks +15 clamped to duration; `ArrowLeft` seeks -15 clamped to 0; no-op without an episode; no-op when `document.hasFocus()` is false; no-op when focus is in an input/textarea/slider.

## 9. Verification

- [x] 9.1 `cargo test` (backend) and `pnpm test`/`pnpm build` (frontend) all pass.
- [x] 9.2 Manual: play an episode, stop at ~20s, reopen it — resumes at the saved point; finish an episode (seek near end) — card shows the played mark and the next episode plays; press start over — position resets.
- [x] 9.3 Manual: with focus on the page, press `ArrowRight`/`ArrowLeft` during playback — seek moves ±15s clamped; typing in the search inputs keeps arrows working as text navigation; pressing arrows with no episode loaded does nothing.

## 10. Rollback note

- [x] 10.1 Verify `down.sql` restores a working `episodes` table before merging (SQLite column drop requires table recreation).