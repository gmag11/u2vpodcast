## Why

Steps 1-3 make the player flow, show what is next, and remember position. The remaining gap for full podcast-client-less listening is the ability to build a persistent, explicitly ordered set of episodes across channels — a playlist — that can be reordered by the user and played through. Storing playlists server-side (per user) makes them follow the user across devices.

## What Changes

- Two new tables: `playlists` (id, user_id, name, created_at, updated_at, unique per user/name) and `playlist_items` (id, playlist_id, episode_id, position, added_at, unique per playlist/episode).
- New authenticated playlist API following the existing `CResponse` + session pattern:
  - `GET`/`POST`/`DELETE /api/1.0/playlists/`
  - `GET`/`POST` `/api/1.0/playlists/{id}/episodes/`
  - `DELETE /api/1.0/playlists/{id}/episodes/{episode_id}/`
  - `PUT /api/1.0/playlists/{id}/episodes/reorder/` with a full ordered list of episode ids.
- New `PlaylistsView` route listing playlist names, opening a playlist shows its ordered episodes.
- `EpisodeCard` gains an "Add to playlist" action (radix-vue dropdown/dialog) listing existing playlists and allowing creation of a new one.
- Playing a playlist seeds the queue (steps 1-2) in playlist order so auto-advance walks the playlist.
- Reordering via up/down controls first; HTML5 drag & drop is a later extension.
- The global History screen and channel screens are unchanged.

## Capabilities

### New Capabilities
- `playlists`: server-persisted, per-user, explicitly ordered playlists of episodes with CRUD, add/remove item, reorder, and playback seeding.

### Modified Capabilities
- `episode-cards`: episode cards expose an "add to playlist" action.

## Impact

- **Code**: `migrations/` (two new tables), `src/models/playlist.rs` (new), `src/handlers/playlists.rs` (new), `src/handlers/mod.rs` (route registration), `frontend/src/lib/api/client.ts` (playlist calls), `frontend/src/router/index.ts` + `frontend/src/stores/` (playlist state), `frontend/src/views/PlaylistsView.vue` (new), `frontend/src/components/EpisodeCard.vue` (add-to-playlist), `frontend/src/i18n/` (en/es strings).
- **APIs**: new `/api/1.0/playlists/` resource (all under the existing `RequireSession` middleware).
- **Dependencies**: none.
- **DB**: two new tables via sqlx migration.
- **Frontend**: playlist state/store, view, and card interaction tests.