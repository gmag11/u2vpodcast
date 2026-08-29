## 1. Backend: schema and model

- [ ] 1.1 Add migration `migrations/2026XXXX0001_add_playback_speed.up.sql`: `ALTER TABLE channels ADD COLUMN playback_speed REAL NOT NULL DEFAULT 1.0`, plus matching `.down.sql` dropping the column
- [ ] 1.2 Add `playback_speed: f64` to the `Channel` struct in `src/models/channel.rs` and read it in `from_row`
- [ ] 1.3 Add `playback_speed` (with `#[serde(default)]`, default 1.0) to the `Episode` struct in `src/models/episode.rs` and read it in `from_row_with_channel`

## 2. Backend: episode payloads carry the channel speed

- [ ] 2.1 Add `COALESCE(c.playback_speed, 1.0) AS playback_speed` to `Episode::read_all_with_channels` and `Episode::read_by_yt_id_with_channel` SQL in `src/models/episode.rs`
- [ ] 2.2 Add the same JOIN column to `PlaylistItem::read_episodes_with_channels` SQL in `src/models/playlist.rs`
- [ ] 2.3 Set `episode.playback_speed = channel.playback_speed` in the channel-episodes handler fill loop (`src/handlers/episodes.rs` `read_with_pagination`, which already sets `channel_slug`)

## 3. Backend: update endpoint

- [ ] 3.1 Add `Channel::set_playback_speed(pool, slug, speed)` in `src/models/channel.rs`: validate finite + within 0.5–3.0, round to two decimals, `UPDATE channels SET playback_speed = $1 WHERE slug = $2 RETURNING *`, not-found error for unknown slug
- [ ] 3.2 Add `PUT /api/1.0/channels/{slug}/playback_speed/` handler in `src/handlers/channels.rs` (body `{ playback_speed: f64 }`, 400 on invalid/out-of-range, 404 unknown slug, 204 on success) and register the route in `src/handlers/mod.rs`
- [ ] 3.3 Backend tests: model `set_playback_speed` (default 1.0, valid update, out-of-range rejected, unknown slug) and handler tests for 200/400/404 paths

## 4. Frontend: types and API client

- [ ] 4.1 Add `playback_speed: number` to `Channel` and `Episode` interfaces in `frontend/src/types.ts`
- [ ] 4.2 Add `setChannelPlaybackSpeed(slug: string, playbackSpeed: number)` to `api` in `frontend/src/lib/api/client.ts` (PUT to `/api/1.0/channels/{slug}/playback_speed/`, expects 204)

## 5. Frontend: player store per-channel speed

- [ ] 5.1 Add `channelSpeedBySlug: Record<string, number>` to the player store (`frontend/src/stores/player.ts`), seeded from every episode's `playback_speed` on load/seed paths
- [ ] 5.2 Add a shared `applyChannelSpeed(episode)` helper setting `speed.value` and `audio.playbackRate` to `episode.playback_speed ?? channelSpeedBySlug[slug] ?? 1.0`, and call it in `loadEpisode` (the funnel for fresh `play`, end-of-episode auto-advance `advance`, manual `skipNext`/`playPrevious`) and in the `togglePlay` reload branch — so every switch to a different channel loads and applies that channel's saved speed and the previous channel's rate is never carried over
- [ ] 5.3 Rework `setSpeed(value)`: clamp to 0.5–3.0, round to two decimals, update `speed.value`/`audio.playbackRate`/MediaSession position state, upsert `channelSpeedBySlug[currentSlug]` when a current episode exists, and fire-and-forget `api.setChannelPlaybackSpeed` (with `.catch` logging, mirroring `persistProgress`)
- [ ] 5.4 Persist `channelSpeedBySlug` in the queue storage: extend `saveQueue`/`loadQueue` in `frontend/src/lib/utils/queue.storage.ts` (and its types) so reloaded sessions restore per-channel speeds

## 6. Frontend: speed control UI

- [ ] 6.1 Redesign the speed panel in `frontend/src/components/PersistentPlayer.vue`: keep preset buttons (0.5x, 1x, 1.25x, 1.5x, 2x), add a current-value label with − and + stepper buttons adjusting ±0.05 within 0.5–3.0, disable steppers at the bounds, show two-decimal labels (e.g. 1.35x), keep the panel open while stepping and close on outside click (existing `data-speed-panel` behavior)
- [ ] 6.2 Add i18n keys in `frontend/src/i18n/locales/en.json` and `es.json` for the speed stepper aria-labels (e.g. speed increase/decrease) if none exist, keeping en/es parity

## 7. Frontend: tests

- [ ] 7.1 Update `frontend/src/stores/player.test.ts`: apply-on-play uses the channel's saved speed; auto-advance (`advance` after the episode ends) into a different channel applies the new channel's speed; manual `skipNext`/`playPrevious` into a different channel applies that channel's speed; the previous channel's rate is never carried over; same-channel skip keeps the speed; setSpeed saves per channel (API called with rounded value); clamp at bounds; restored-queue speed fallback
- [ ] 7.2 Update `frontend/src/components/PersistentPlayer.test.ts`: stepper ±0.05 adjustments, presets still selectable, bounds disable steppers, displayed label formatting

## 8. Verification

- [ ] 8.1 Run backend tests (`cargo test`) and frontend unit tests (`npm test` in `frontend/`), fix any failures
- [ ] 8.2 Re-run `openspec validate --change playback-speed-per-channel` (or the repo's spec validation) to confirm the change artifacts are valid