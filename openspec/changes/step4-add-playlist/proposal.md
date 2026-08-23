## Why

Steps 1-3 give the player flow, a visible queue, and remembered progress. Steps 4's original design (multiple named playlists) was over-engineered for a podcast app: a listener needs one clear "up next / pending" list. The playlist is now a single per-user ordered list of episodes; finishing an episode removes it from the list and marks it listened; a previously played episode can be put back as pending.

## What Changes

- **Single playlist per user** (no playlist names, no playlist CRUD). New table `playlist_items (user_id, episode_id, position, added_at)` with `UNIQUE(user_id, episode_id)`.
- New authenticated singular API:
  - `GET /api/1.0/playlist/` — ordered items joined with channel slug/title.
  - `POST /api/1.0/playlist/` — append an episode at the end (`{ episode_id }`); duplicate → conflict.
  - `DELETE /api/1.0/playlist/{episode_id}/` — remove an episode and reindex.
  - `PUT /api/1.0/playlist/reorder/` — full ordered list of episode ids rewrites positions.
- Playing a playlist episode seeds the queue (steps 1-2) in playlist order.
- When an episode finishes (`ended` or the step-2 long-press skip that marks it listened), it is removed from the playlist server-side; short-press skip does not (it does not mark listened).
- New "mark as not listened" control on episode cards: sets `listen=false` (position reset to 0) and re-appends the episode to the end of the playlist.
- The History screen and channel screens are unchanged.

## Capabilities

### New Capabilities
- `playlist`: a single ordered, server-persisted, per-user playlist (pending episodes) with add, remove, reorder, completion removal, and playback seeding.

### Modified Capabilities
- `episode-cards`: cards gain an add/remove toggle for the single playlist and (via step 3) the ability to unmark an episode as listened.

## Impact

- **Code**: `migrations/` (one new table), `src/models/playlist.rs` (new), `src/handlers/playlists.rs` (new, no playlist-name CRUD), `src/handlers/mod.rs` (route registration), `frontend/src/lib/api/client.ts`, `frontend/src/router/index.ts`, `frontend/src/stores/playlists.ts`, `frontend/src/views/PlaylistView.vue` (new), `frontend/src/components/EpisodeCard.vue`, `frontend/src/i18n/` (en/es).
- **APIs**: new singular `/api/1.0/playlist/` resource under `RequireSession`.
- **Dependencies**: none.
- **DB**: one new table via sqlx migration.
- **Frontend**: playlist store, view, card toggle, unmark flow.