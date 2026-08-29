## Context

The app is a Rust backend (actix-web 4, sqlx/SQLite) serving a Vue 3 SPA. Playback is driven by a single shared `player` Pinia store owning one `<audio>` element; a persistent bottom bar exposes the controls. Today the speed control is a fixed-preset dropdown (`[0.5, 1, 1.25, 1.5, 2]`) rendered in `PersistentPlayer.vue`, and the store's `setSpeed(value)` only mutates the in-memory `speed` ref and `audio.playbackRate`. Speed is never persisted anywhere: after a reload the rate always returns to 1x, and there is no notion of a speed belonging to a channel. Episode payloads already carry `channel_slug`, and playback-progress persistence already follows the server-side fire-and-forget pattern this change mirrors.

## Goals / Non-Goals

**Goals:**
- Let the user select any playback rate in 0.05 steps (e.g. 1.35x, 1.7x) via +/− stepper controls in the speed panel, alongside the existing presets.
- Persist one playback speed per channel server-side so it applies to every episode of the channel, survives reloads, and is shared across devices.
- Start every episode at its channel's saved speed; any manual change overwrites the saved value for that channel.
- Keep the change small and consistent with existing patterns (server-side persistence, fire-and-forget saves, single shared store).

**Non-Goals:**
- Per-episode speed overrides (the requirement is explicitly per-channel).
- Global (non-channel) speed preference.
- Speed presets removal — presets stay as quick-selects.
- Version bump / release tooling (no user-visible version change needed for this feature).

## Decisions

### D1: Store the speed server-side as a column on `channels`

A new migration adds `playback_speed REAL NOT NULL DEFAULT 1.0` to `channels`. The `Channel` struct and `from_row` gain the field so every channels API payload includes it automatically.

**Alternatives considered:**
- *Frontend `localStorage` map*: simplest, but not shared across devices and inconsistent with how progress is already persisted server-side.
- *Column on `episodes`*: fragments the preference (every episode would need syncing); the requirement is channel-scoped.

Rationale: a single per-channel column matches "saved per channel" exactly, gives a canonical server-side value, and existing rows default to 1.0 with a plain `NOT NULL DEFAULT` migration (same style as previous channel migrations, e.g. `add_slug`, `add_sync_status`).

### D2: Deliver the channel speed through episode payloads (no extra request at play time)

Extend the episode SQL builders (`read_all_with_channels`, `read_by_yt_id_with_channel`) with `COALESCE(c.playback_speed, 1.0) AS playback_speed` and the channel-episodes handler fill loop (which already sets `episode.channel_slug = channel.slug`) with the channel's speed. The `Episode` struct gains `#[serde(default)] pub playback_speed: f64` (default 1.0) so older payloads stay resolvable.

Rationale: playback must apply the speed the moment an episode starts; carrying it on the episode object means zero latency and no extra round-trip, mirroring how `channel_slug` already travels on episodes. The channels list (which also gains the field via D1) stays the source for channel-level consumers like the settings/edit UI.

### D3: Fine-grained stepper in the speed panel

Redesign the dropdown content of the speed control in `PersistentPlayer.vue`:
- a current-value label (`1.35x`) with − and + buttons stepping `±0.05`;
- the existing preset buttons as quick-selects;
- clamping to the supported range `0.5`–`3.0` at both ends (stepper buttons disable at the bounds);
- each step calls `setSpeed` immediately so the rate change is audible/visible at once, and the panel stays open while stepping (it still closes on outside click, reusing the existing `data-speed-panel` outside-click handling).

Display and stored values are normalized to two decimals (round-half-up) to avoid float artifacts like `1.7000000000000002`.

Rationale: `±0.05` matches the user's stated granularity ("half-tenths", e.g. 1.35 or 1.7); presets remain one-tap; immediate application keeps feedback live. Bounds 0.5–3.0 cover the requested values with comfortable headroom and stay within typical browser `playbackRate` support.

### D4: Store-side semantics: apply on play, save on change

- The player store keeps `channelSpeedBySlug: Record<slug, number>` seeded from every episode payload's `playback_speed` (in `play()`/`loadEpisode()`/`seedProgress`-style seeding of fetched lists).
- A single `applyChannelSpeed(episode)` helper resolves `episode.playback_speed ?? channelSpeedBySlug[slug] ?? 1.0` and writes both `speed.value` and `audio.playbackRate`. It is invoked on **every** source-loading path: `loadEpisode()` — the funnel shared by fresh `play`, the end-of-episode auto-advance (`advance`), manual `skipNext`/`playPrevious`, and repeat-all replays — and the `togglePlay` reload branch (restored-queue restart loads the source inline, not via `loadEpisode`). Because `audio.playbackRate` is a persistent element property that survives `src` changes, re-applying it on every switch is what guarantees a cross-channel change loads and applies the **new** channel's value and never inherits the previous channel's rate.
- The persisted queue (`queue.storage`) additionally stores the `channelSpeedBySlug` map, so a reloaded session (whose restored episodes carry no payload fields) still starts at the right speed; unknown slugs fall back to 1.0.
- `setSpeed(value)` becomes the single write path: it clamps+rounds, updates `speed.value`/`audio.playbackRate`/MediaSession position state, upserts `channelSpeedBySlug[channelSlug]` (when a current episode exists), and fires a fire-and-forget `PUT` to the new endpoint — exactly the pattern of `persistProgress` (including `.catch` logging). No debounce: each press is a user gesture and last-write-wins is acceptable (same as progress saves).

Rationale: reusing the existing "apply on load / fire-and-forget save" shape keeps the store coherent; persisting the map in the queue storage closes the reload gap without blocking play on a network lookup (an alternative — `armResume`-style lookup — was rejected because it would add latency exactly when the user expects playback to start).

### D5: New update endpoint

`PUT /api/1.0/channels/{slug}/playback_speed/` with body `{ "playback_speed": number }`:
- rejects non-finite values and values outside `0.5`–`3.0` with 400; unknown slug with 404;
- rounds valid values to two decimals, writes `channels.playback_speed`, answers 204 (matching the existing progress/favorite update endpoints and the client's `request<null>` handling);
- `Channel` model offers `set_playback_speed(pool, slug, speed)`.

Rationale: 204 mirrors sibling endpoints (`updateEpisodeProgress`, `setEpisodeFavorite`) — the client already treats 204 as "no data". Server-side validation + rounding makes the stored value canonical regardless of client behavior.

## Risks / Trade-offs

- [Browser playbackRate clamping varies (some engines clamp outside ~0.5–4)] → Mitigation: UI and API enforce 0.5–3.0, well inside common limits; the browser's effective rate is accepted as final (audio element is authoritative for playback).
- [Stale playback rate leaking across channel switches — the element keeps its `playbackRate` when the source changes, so a switch to a channel with a different saved speed could keep playing at the old rate] → Mitigation: the `applyChannelSpeed` helper runs on every source-load path (auto-advance, manual skip, play, restore) and cross-channel tests assert the new channel's rate is applied and the old one is dropped.
- [Float representation artifacts (1.7 → 1.7000000000000002)] → Mitigation: round to two decimals on store write, API write, and display; tests assert on rounded values.
- [Concurrent writes from another device] → Mitigation: last-write-wins, identical to existing progress semantics; acceptable for a per-channel preference.
- [Reloaded queue holds episodes without payload speed] → Mitigation: `channelSpeedBySlug` persisted in queue storage plus 1.0 fallback; any later fetch of the same channel refreshes the map.
- [Adding a column to `channels` requires migration ordering care] → Mitigation: plain additive `REAL NOT NULL DEFAULT 1.0` (like prior channel migrations); no data backfill needed; rollback is a drop column migration.

## Migration Plan

1. Add migration `2026XXXX0001_add_playback_speed.up.sql` (`ALTER TABLE channels ADD COLUMN playback_speed REAL NOT NULL DEFAULT 1.0`) and matching `.down.sql` (drop column).
2. Deploy backend + frontend together (static SPA served by the same process); the additive column and new endpoint are backward compatible, so a rolling deploy with the old frontend briefly missing the field is harmless (default 1.0).
3. Rollback: revert the column migration; frontend falls back to presets + 1.0 default with no functional breakage (saved speeds are lost).

## Open Questions

- None blocking implementation. Minor: whether the +/− stepper should also allow hold-to-repeat; deferred — click-per-step is the v1 behavior and can be added later without spec changes.