## 1. Migration

- [ ] 1.1 Create `migrations/<timestamp>_add_playlist.up.sql` with `playlist_items` (id, episode_id, position, added_at, `UNIQUE(episode_id)`). No `user_id` (single-user app).
- [ ] 1.2 Create the matching `.down.sql` dropping the table.
- [ ] 1.3 Confirm the migration applies on a fresh memory DB (`cargo test` Migrator) and on the dev DB.

## 2. Backend model

- [ ] 2.1 Create `src/models/playlist.rs`: `PlaylistItem` struct with `from_row` helpers following `channel.rs`/`episode.rs` style.
- [ ] 2.2 Implement `read_all(pool) -> items ordered by position` and `read_episodes_with_channels(pool)` joining `episodes` + `channels` (channel_slug/channel_title) ordered by `position ASC`, returning `Episode` values.
- [ ] 2.3 Implement `add(pool, episode_id)`: position = `max(position)+1`; duplicate (UNIQUE conflict) surfaces an error mapped to conflict.
- [ ] 2.4 Implement `remove(pool, episode_id)`: delete the item and reindex remaining positions contiguously.
- [ ] 2.5 Implement `reorder(pool, episode_ids: &[i64])`: validate the submitted set matches the stored set, rewrite positions in the given order (missing episodes omitted).

## 3. Backend handlers + routes

- [ ] 3.1 Create `src/handlers/playlists.rs` with `pub fn api_playlists(cfg)`; no `from_session` ownership checks (global resource).
- [ ] 3.2 Routes: `GET /playlist/`, `POST /playlist/` (`{ episode_id }`, conflict on duplicate), `DELETE /playlist/{episode_id}/`, `PUT /playlist/reorder/` (`{ episode_ids }`).
- [ ] 3.3 Respond with `CResponse::ok(session, ...)`; conflict → `CResponse::ko(StatusCode::CONFLICT, session)`; unknown episode → 404.
- [ ] 3.4 Mount `.configure(playlists::api_playlists)` in `src/handlers/mod.rs` inside the `RequireSession` scope and add `mod playlists;`.

## 4. Frontend API client

- [ ] 4.1 In `frontend/src/lib/api/client.ts` add: `getPlaylist()`, `addEpisodeToPlaylist(episodeId)`, `removeEpisodeFromPlaylist(episodeId)`, `reorderPlaylist(episodeIds)`.
- [ ] 4.2 In `frontend/src/types.ts` no new types needed (episodes already typed; item list is `Array<Episode>` in position order).

## 5. Frontend playlist store

- [ ] 5.1 Create `frontend/src/stores/playlists.ts`: state `items: Episode[]` and `episodeIdSet` (computed set); actions `load()`, `add(episodeId)`, `remove(episodeId)`, `reorder(episodeIds)` calling `api` and updating state.
- [ ] 5.2 `add` skips when the episode id is already present (defensive); `remove` reconciles after `load()`.

## 6. Route, header, view

- [ ] 6.1 Add `/playlist` route and header navigation (desktop + mobile drawer) in `AppHeader.vue` mirroring the History entry.
- [ ] 6.2 Create `frontend/src/views/PlaylistView.vue`: ordered `EpisodeCard`s with up/down reorder arrows, remove per item, play-all / per-card play (seeds queue), and an empty state ("no pending episodes").
- [ ] 6.3 Session guard exactly like `EpisodesView.vue`/`HistoryView.vue` (redirect to login when session missing).

## 7. Card toggle and unmark

- [ ] 7.1 In `frontend/src/components/EpisodeCard.vue` add a playlist toggle button reflecting `playlists.episodeIdSet`: add when absent, remove when present; notification via the existing notification store on both outcomes.
- [ ] 7.2 Add the "mark as not listened" control when `props.episode.listen` is true: call `updateEpisodeProgress(id, { position_seconds: 0, listened: false })`, then `playlists.add(id)` (re-append at end); refresh the card's in-memory episode so the played mark clears immediately.
- [ ] 7.3 If the playlist add fails, still keep the cleared listened state and surface the error notification.

## 8. i18n

- [ ] 8.1 Add en/es strings for the playlist (title, empty state, add/remove notifications, unmark, reorder tooltips) and keep locale parity (parity test).

## 9. Completion removal from playlist

- [ ] 9.1 In `frontend/src/stores/player.ts` add `queueSource: 'playlist' | 'list'` seeded when `play(episode, list)` is called from the playlist view; default `'list'` for other views.
- [ ] 9.2 In `markListened()` (step 3) and `onEnded`/long-press completion: when the finished episode came from the playlist source, also call `playlists.remove(episodeId)` (fire-and-forget; 404 ignored). Short-press skip does not mark listened and does not remove.

## 10. Tests

- [ ] 10.1 Backend (`src/models/playlist.rs` tests, memory-pool pattern): add appends and rejects duplicates; remove reindexes; reorder rewrites positions; episode join returns channel slug/title.
- [ ] 10.2 Frontend (`frontend/src/stores/playlists.test.ts`): store actions update items and set; queue seeding from playlist order.
- [ ] 10.3 Card component test: toggle renders add vs remove per `episodeIdSet`; unmark calls progress + add.

## 11. Verification

- [ ] 11.1 `cargo test`, `pnpm test`, `pnpm build` all pass.
- [ ] 11.2 Manual: add episodes from cards; reorder with up/down; play the playlist and confirm auto-advance walks it while each finished episode leaves the playlist; long-press next removes + marks; short-press keeps it; unmark re-appends at the end; empty state renders.

## 12. Rollback note

- [ ] 12.1 Verify `down.sql` drops the table cleanly before merging.