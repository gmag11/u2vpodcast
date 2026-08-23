## 1. Migration

- [ ] 1.1 Create `migrations/<timestamp>_add_playlists.up.sql` with `playlists` (id, user_id, name, created_at, updated_at, `UNIQUE(user_id, name)`) and `playlist_items` (id, playlist_id, episode_id, position, added_at, `UNIQUE(playlist_id, episode_id)`).
- [ ] 1.2 Create the matching `.down.sql` dropping both tables.
- [ ] 1.3 Confirm the migration applies on a fresh memory DB (`cargo test` migrator) and on the dev DB.

## 2. Backend model

- [ ] 2.1 Create `src/models/playlist.rs`: `Playlist` (with `created_at`/`updated_at`), `PlaylistRead` (adds `episode_count`), and item structs with `from_row` helpers following `channel.rs`/`episode.rs` style.
- [ ] 2.2 Implement `list_for_user(pool, user_id)`, `create(pool, user_id, name)`, `rename(pool, id, user_id, name)`, `delete_with_items(pool, id, user_id)`.
- [ ] 2.3 Implement `read_episodes_for(pool, playlist_id, user_id)` joining `episodes` + `channels` (channel_slug/channel_title) ordered by `position ASC`; returns `Episode` values.
- [ ] 2.4 Implement `add_episode` (position = `max(position)+1`; duplicate → conflict), `remove_episode` (delete + reindex positions contiguously), `reorder(pool, playlist_id, episode_ids: &[i64])` (validate set membership, rewrite positions).
- [ ] 2.5 Add owner-scoping to every query (`AND user_id = ?` on playlist reads; step 3 exposes the id).

## 3. Backend handlers + routes

- [ ] 3.1 Create `src/handlers/playlists.rs` with `pub fn api_playlists(cfg)` registering the routes below; resolve `user_id` from `models::user::from_session(&session)`.
- [ ] 3.2 Routes: `GET /playlists/` (list with counts), `POST /playlists/` (`{ name }`, conflict on duplicate), `PUT /playlists/{id}/` (`{ name }` rename), `DELETE /playlists/{id}/` (delete + items).
- [ ] 3.3 Routes: `GET /playlists/{id}/episodes/`, `POST /playlists/{id}/episodes/` (`{ episode_id }`), `DELETE /playlists/{id}/episodes/{episode_id}/`, `PUT /playlists/{id}/episodes/reorder/` (`{ episode_ids }`).
- [ ] 3.4 Register `openapi`-style responses via `CResponse::ok/ko`; unknown/non-owned ids return 404.
- [ ] 3.5 Mount `.configure(playlists::api_playlists)` in `src/handlers/mod.rs` inside the `RequireSession` scope and add `mod playlists;`.

## 4. Frontend API client

- [ ] 4.1 In `frontend/src/lib/api/client.ts` add playlist methods: `getPlaylists()`, `createPlaylist(name)`, `renamePlaylist(id, name)`, `deletePlaylist(id)`, `getPlaylistEpisodes(id)`, `addEpisodeToPlaylist(id, episodeId)`, `removeEpisodeFromPlaylist(playlistId, episodeId)`, `reorderPlaylist(playlistId, episodeIds)`.
- [ ] 4.2 Add `Playlist` type (id, name, episode_count, created_at, updated_at) to `frontend/src/types.ts`.

## 5. Frontend playlist store

- [ ] 5.1 Create `frontend/src/stores/playlists.ts`: state `playlists`, `active` (id + episodes); actions `load()`, `create(name)`, `rename(id, name)`, `remove(id)`, `addEpisode(playlistId, episodeId)`, `removeEpisode(playlistId, episodeId)`, `reorder(playlistId, episodeIds)` calling `api` and updating local state.
- [ ] 5.2 Add `player.play(episode, playlistEpisodes)` seeding when playing from a playlist detail (reuses step 1-2 queue mechanics).

## 6. Routes, header, view

- [ ] 6.1 Add `/playlists` route (list) and `/playlists/:id` (detail) in `frontend/src/router/index.ts`; wire header navigation (desktop + mobile drawer) in `AppHeader.vue` mirroring the History entry.
- [ ] 6.2 Create `frontend/src/views/PlaylistsView.vue`: list playlists with name, episode count, rename/delete actions; empty state.
- [ ] 6.3 Create the playlist detail view: `EpisodeCard` list in stored order with up/down reorder arrows, remove item, "play all"/per-card play (seeds queue), back navigation.
- [ ] 6.4 Handle load/session guards exactly like `EpisodesView.vue`/`HistoryView.vue` (redirect to login when session missing).

## 7. Card add-to-playlist action

- [ ] 7.1 In `frontend/src/components/EpisodeCard.vue` add a playlist dropdown (radix-vue, pattern from `AppHeader.vue`): lists existing playlists + "New playlist…" inline input.
- [ ] 7.2 Wire add: call `playlists.addEpisode` (or `create`+`add`), show success via the notification store; on duplicate, show the error message.
- [ ] 7.3 Guard when no playlists exist: show a prominent "Create playlist" primary action in the dropdown.

## 8. i18n

- [ ] 8.1 Add en/es strings for playlists (title, new, rename, delete, add-to-playlist, already-added, empty states, reorder) and keep locale parity (parity test).

## 9. Tests

- [ ] 9.1 Backend (`src/models/playlist.rs` tests, memory-pool pattern): create/list/rename/delete scoped by user; add appends and rejects duplicates; remove reindexes; reorder rewrites positions; episode join returns channel slug/title.
- [ ] 9.2 Frontend (`frontend/src/stores/playlists.test.ts`): store actions update state; queue seeding from playlist order.
- [ ] 9.3 Component test for the card dropdown add flow if feasible with `@vue/test-utils`.

## 10. Verification

- [ ] 10.1 `cargo test`, `pnpm test`, `pnpm build` all pass.
- [ ] 10.2 Manual: create/rename/delete playlists; add episodes from cards; reorder with up/down; play a playlist and confirm auto-advance walks it in order; confirm duplicates are rejected with a message; confirm another browser session (same login) sees the same playlists.

## 11. Rollback note

- [ ] 11.1 Verify `down.sql` drops both tables cleanly before merging.