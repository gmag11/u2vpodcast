## Context

The backend is Actix-web + sqlx/SQLite. Existing patterns:
- Models: plain structs with `from_row(SqliteRow)` helpers and CRUD methods (`src/models/{channel,episode,user}.rs`).
- Handlers: module with `pub fn api_<name>(cfg: &mut ServiceConfig)` registering `#[get]/#[post]/#[put]/#[delete]` routes (`src/handlers/users.rs`), mounted in `src/handlers/mod.rs` under the `RequireSession` scope.
- Responses: `CResponse::ok(session, data)`; the session user id comes from `models::user::from_session(&session)` → `SessionUser { id, .. }`.
- Migrations: numbered `.up.sql`/`.down.sql` files in `migrations/`, run by the built-in sqlx `Migrator` at startup.

Frontend: Vue 3 + Pinia + `radix-vue` (dropdowns already used in `AppHeader.vue`), `vue-router` with routes registered in `frontend/src/router/index.ts`, i18n en/es in `frontend/src/i18n/locales/`.

## Goals / Non-Goals

**Goals:**
- Server-persisted, per-user, named playlists of episodes with an explicit order.
- CRUD (create/list/delete), add/remove episode, and full reorder.
- Playlist playback seeds the queue in playlist order (auto-advance steps 1-2 walk it).
- Add-to-playlist surface on episode cards.

**Non-Goals:**
- Drag & drop reorder (up/down controls first; DnD is a later extension).
- Shared/collaborative playlists, import/export, covers/thumbnails for playlists.
- Renaming is included as a simple `PUT`; no playlist metadata beyond name.

## Decisions

### Decision 1: Schema

```sql
CREATE TABLE IF NOT EXISTS playlists (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    user_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL,
    UNIQUE(user_id, name)
);
CREATE TABLE IF NOT EXISTS playlist_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    playlist_id INTEGER NOT NULL,
    episode_id INTEGER NOT NULL,
    position INTEGER NOT NULL,
    added_at DATETIME NOT NULL,
    UNIQUE(playlist_id, episode_id)
);
```

No foreign-key constraints, consistent with the existing `episodes.channel_id` approach; referential integrity is enforced in handlers (playlist/episode existence checked before insert, items deleted when the playlist is deleted). `position` is 0-based and rewritten on every reorder/removal.

**Why**: the existing schema avoids FK constraints (see `episodes`), and handler-enforced integrity keeps migrations simple.

### Decision 2: API surface (all under `RequireSession`, `CResponse` envelope)

| Method | Path | Behavior |
|---|---|---|
| GET | `/api/1.0/playlists/` | list playlists for the session user (+ item count) |
| POST | `/api/1.0/playlists/` | `{ name }` → create (409 on duplicate name) |
| PUT | `/api/1.0/playlists/{id}/` | `{ name }` → rename |
| DELETE | `/api/1.0/playlists/{id}/` | delete playlist + its items |
| GET | `/api/1.0/playlists/{id}/episodes/` | episodes ordered by `position ASC`, joined with channels (slug/title) |
| POST | `/api/1.0/playlists/{id}/episodes/` | `{ episode_id }` → append at `max(position)+1` (409/400 if already present) |
| DELETE | `/api/1.0/playlists/{id}/episodes/{episode_id}/` | remove item, reindex positions |
| PUT | `/api/1.0/playlists/{id}/episodes/reorder/` | `{ episode_ids: [...] }` → validate same set, rewrite positions in given order |

Ownership: every handler resolves `user_id` from `from_session(&session)?.id` and scopes queries to it; non-owned ids return 404.

**Why**: mirrors `users.rs`/`channels.rs` style; a single `playlists.rs` module keeps the resource cohesive. Reorder-as-full-list is unambiguous and naturally fills a full reorder UI.

### Decision 3: Rust model `src/models/playlist.rs`

`Playlist` and `PlaylistRead` (with `episode_count`) plus `PlaylistItem` structs; methods: `list_for_user`, `create`, `rename`, `delete_with_items`, `read_episodes_for` (JOIN episodes+channels), `add_episode`, `remove_episode` (reindex), `reorder`. Tests follow the existing `episode_update_tests` memory-pool pattern.

**Why**: one cohesive model module per resource, matching `channel.rs`/`user.rs`.

### Decision 4: Frontend state and routes

- Pinia store `playlists.ts`: `list`, `create`, `rename`, `delete`, `addEpisode`, `removeEpisode`, `reorder` (calls `api` and keeps the active playlist cache).
- `api` client methods mirroring the endpoints in `frontend/src/lib/api/client.ts`.
- New routes in `frontend/src/router/index.ts`: `/playlists` (list + detail) and per-playlist detail; header entry alongside History.
- `PlaylistsView.vue`: name list with delete; opening one shows ordered episodes with up/down reorder, remove, and play (seed queue).

**Why**: a dedicated store keeps playlist data reactive across the card dropdown and the view without refetch storms; header nav follows the existing History nav pattern.

### Decision 5: Add-to-playlist on episode cards

`EpisodeCard` gains a dropdown (radix-vue `DropdownMenuRoot`, pattern from `AppHeader.vue`): "Añadir a playlist" → existing playlists + "New playlist…" inline input creating then adding. Add confirmation via the existing notification store.

**Why**: cards are the shared episode surface (Channels/Episodes/History views); reusing radix-vue keeps behavior consistent.

### Decision 6: Playback seeding

Playing an episode from a playlist calls `player.play(episode, playlistEpisodes)` — the queue (steps 1-2) becomes the remaining playlist order, so auto-advance walks the playlist. The queue remains localStorage; playlists remain server-side (queue = transient working copy, playlist = durable source).

**Why**: separation matches the agreed persistence split (queue localStorage, playlists server); no new player mechanics needed.

## Risks / Trade-offs

- **[Risk] Concurrent reorders by the same user across tabs.** Single-user deployments make this negligible; last-write-wins accepted.
- **[Risk] Episodes deleted/unsynced while referenced by a playlist.** `read_episodes_for` uses an INNER JOIN and the frontend tolerates gaps; stale items are removed lazily on next access.
- **[Trade-off] Up/down reorder is O(n) taps for long playlists.** Accepted for accessibility/simplicity; DnD extension noted as follow-up.

## Migration Plan

1. Add migration for the two tables.
2. Backend: `playlist.rs` model + `playlists.rs` handlers + route registration (`config_services`).
3. Frontend: API client, Pinia store, routes, `PlaylistsView`, card dropdown.
4. i18n strings (en/es).
5. Tests: Rust model/handler (memory pool), Vitest store/queue-seeding; manual verification per `tasks.md`.

**Rollback**: revert code; `down.sql` drops both tables.

## Open Questions

None.